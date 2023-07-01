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

use std::collections::HashMap;
use url::Url;
mod url_lua;

#[derive(Clone)]
pub struct HttpMessage {
    pub url: Option<Url>,
}

impl HttpMessage {
    // This method takes a payload string and a boolean indicating whether or not to remove the content of the query parameter
    // It returns a HashMap of new URLs with the modified query parameter and corresponding value
    pub fn change_urlquery(&self, payload: &str, remove_content: bool) -> HashMap<String, String> {
        // Create two HashMaps to store the original and modified query parameters
        let mut scan_params = HashMap::with_capacity(16);
        let mut result = HashMap::with_capacity(16);

        // If there is no URL, return an empty HashMap
        if let Some(the_url) = &self.url {
            // If there is a URL, add its query parameters to scan_params
            for (key, value) in the_url.query_pairs() {
                scan_params.insert(key.to_string(), value.to_string());
            }
        } else {
            return result;
        }

        // For each query parameter in scan_params, split the payload string by newlines and create a new URL with each modified query parameter
        for (key, value) in scan_params.iter() {
            for pl in payload.split('\n') {
                let mut new_params = scan_params.clone();
                if remove_content {
                    new_params.insert(key.to_string(), pl.to_string());
                } else {
                    let mut new_value = String::with_capacity(value.len() + pl.len());
                    new_value.push_str(value);
                    new_value.push_str(pl);
                    new_params.insert(key.to_string(), new_value);
                }

                let mut new_url = self.url.clone().unwrap();
                new_url.set_query(None);

                for (key, value) in new_params.iter() {
                    new_url.query_pairs_mut().append_pair(key, value);
                }

                result.insert(key.to_string(), new_url.as_str().to_string());
            }
        }

        result
    }

    // This method takes a query parameter, a payload string, and a boolean indicating whether or not to remove the content of the query parameter
    // It returns a new URL with the modified query parameter and corresponding value
    pub fn set_urlvalue(&self, param: &str, payload: &str, remove_content: bool) -> String {
        // If there is no URL, return an empty String
        if let Some(mut url) = self.url.clone() {
            // If there is a URL, modify the specified query parameter and return the new URL as a String
            let new_query = url
                .query_pairs()
                .fold(String::new(), |mut acc, (key, value)| {
                    if key == param {
                        if remove_content {
                            acc.push_str(&format!("{}={}", key, payload));
                        } else {
                            let new_value = format!("{}{}", value, payload);
                            acc.push_str(&format!("{}={}", key, new_value));
                        }
                    } else {
                        acc.push_str(&format!("{}={}", key, value));
                    }
                    acc.push('&');
                    acc
                });
            url.set_query(Some(&new_query[..new_query.len() - 1]));
            return url.as_str().to_string();
        }
        String::new()
    }

    pub fn urljoin(&self, path: &str) -> String {
        if let Some(url) = &self.url {
            return url.join(path).unwrap().as_str().to_string();
        }
        String::new()
    }
}
