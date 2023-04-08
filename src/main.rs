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
    match Opts::from_args() {
        Opts::SCAN { .. } => run_scan().await,
        Opts::NEW {
            scan_type,
            file_name,
        } => {
            new_args(scan_type, file_name);
            std::process::exit(0);
        }
    }
}

async fn run_scan() -> Result<(), std::io::Error> {
    let opts = args_scan();
    let fuzz_workers = opts.fuzz_workers;
    show_msg(
        &format!("URLS: {}", opts.target_data.urls.len()),
        MessageLevel::Info,
    );
    show_msg(
        &format!("HOSTS: {}", opts.target_data.hosts.len()),
        MessageLevel::Info,
    );
    show_msg(
        &format!("PATHS: {}", opts.target_data.paths.len()),
        MessageLevel::Info,
    );
    show_msg(&format!("CUSTOM: {}", opts.target_data.custom.len()), 
        MessageLevel::Info);
    // Open two threads for URL/HOST scanning
    create_progress(opts.target_data.urls.len() as u64);
    {
        *SLEEP_TIME.lock().unwrap() = opts.delay;
        *REQUESTS_LIMIT.lock().unwrap() = opts.requests_limit;
        *VERBOSE_MODE.lock().unwrap() = opts.verbose;
    }
    {
        BAR.lock().unwrap().suspend(|| {})
    };
    let scan_futures = vec![
        opts.lotus_obj.start(
            opts.target_data.paths,
            None,
            opts.req_opts.clone(),
            ScanTypes::PATHS,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            opts.target_data.urls,
            None,
            opts.req_opts.clone(),
            ScanTypes::URLS,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            vec![],
            Some(opts.target_data.custom),
            opts.req_opts.clone(),
            ScanTypes::CUSTOM,
            opts.exit_after,
            fuzz_workers,
        ),
        opts.lotus_obj.start(
            opts.target_data.hosts,
            None,
            opts.req_opts,
            ScanTypes::HOSTS,
            opts.exit_after,
            fuzz_workers,
        ),
    ];
    runner::scan_futures(scan_futures, 4, None).await;
    BAR.lock().unwrap().finish();
    Ok(())
}
