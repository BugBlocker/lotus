// This file is part of Lotus Project, a web security scanner written in Rust based on Lua scripts.
// For details, please see https://github.com/rusty-sec/lotus/
//
// Copyright (c) 2022 - Khaled Nassar
//
// Please note that this file was originally released under the GNU General Public License as
// published by the Free Software Foundation; either version 2 of the License, or (at your option)
// any later version.
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
// either express or implied. See the License for the specific language governing permissions
// and limitations under the License.

use lotus::{
    cli::{
        args::Opts,
        bar::{create_progress, show_msg, MessageLevel, BAR},
        input::load_scripts::get_scripts,
        startup::{new::new_args, scan::scan::args_scan},
    },
    lua::{
        network::http::{REQUESTS_LIMIT, SLEEP_TIME, VERBOSE_MODE},
        threads::runner,
    },
    ScanTypes,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if let Opts::NEW(new_opts) = Opts::from_args() {
        new_args(new_opts.scan_type, new_opts.file_name);
        std::process::exit(0);
    }
    run_scan().await.unwrap();
    Ok(())
}

async fn run_scan() -> Result<(), std::io::Error> {
    // Spawn a new thread to handle the exit process when the user presses CTRL + C.
    runner::pause_channel().await;
    let opts = args_scan();
    let scripts = get_scripts(opts.lotus_obj.script_path.clone());
    let fuzz_workers = opts.fuzz_workers;
    show_msg(
        &format!("Number of URLs: {}", opts.target_data.urls.len()),
        MessageLevel::Info,
    );

    show_msg(
        &format!("Number of hosts: {}", opts.target_data.hosts.len()),
        MessageLevel::Info,
    );

    show_msg(
        &format!(
            "Number of HTTP MSGS: {}",
            opts.target_data.parse_requests.len()
        ),
        MessageLevel::Info,
    );
    show_msg(
        &format!("Number of paths: {}", opts.target_data.paths.len()),
        MessageLevel::Info,
    );

    show_msg(
        &format!(
            "Number of custom entries: {}",
            opts.target_data.custom.len()
        ),
        MessageLevel::Info,
    );
    // Open two threads for URL/HOST scanning
    create_progress(
        (opts.target_data.hosts.len()
            + opts.target_data.paths.len()
            + opts.target_data.custom.len()
            + opts.target_data.urls.len() * scripts.len()) as u64,
    );
    {
        *SLEEP_TIME.lock().await = opts.delay;
        *REQUESTS_LIMIT.lock().await = opts.requests_limit;
        *VERBOSE_MODE.lock().await = opts.verbose;
    }
    let scan_futures = vec![
        opts.lotus_obj.start(
            opts.target_data.parse_requests,
            scripts.clone(),
            opts.req_opts.clone(),
            ScanTypes::FULL_HTTP,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            convert_serde_value(opts.target_data.paths),
            scripts.clone(),
            opts.req_opts.clone(),
            ScanTypes::PATHS,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            convert_serde_value(opts.target_data.urls),
            scripts.clone(),
            opts.req_opts.clone(),
            ScanTypes::URLS,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            convert_serde_value(opts.target_data.hosts),
            scripts.clone(),
            opts.req_opts.clone(),
            ScanTypes::HOSTS,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            opts.target_data.custom,
            scripts.clone(),
            opts.req_opts,
            ScanTypes::CUSTOM,
            opts.exit_after,
            fuzz_workers,
        ),
    ];
    runner::scan_futures(scan_futures, 4, None).await;
    BAR.lock().unwrap().finish();
    Ok(())
}

fn convert_serde_value(data: Vec<String>) -> Vec<serde_json::Value> {
    data.into_iter()
        .map(|s| serde_json::Value::String(s))
        .collect()
}
