// Copyright 2018-2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod chain_type;

use ccore::{MemPoolFees, MinerOptions, TimeGapParams};
use cidr::IpCidr;
use ckey::PlatformAddress;
use clap;
use cnetwork::{FilterEntry, NetworkConfig, SocketAddr};
use primitives::H256;
use std::fs;
use std::str::{self, FromStr};
use std::time::Duration;
use toml;

pub use self::chain_type::ChainType;
use crate::rpc::{RpcHttpConfig, RpcIpcConfig, RpcWsConfig};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ipc: Ipc,
    #[serde(rename = "codechain")]
    pub operating: Operating,
    pub mining: Mining,
    pub network: Network,
    pub rpc: Rpc,
    pub ws: Ws,
    pub snapshot: Snapshot,
    #[serde(default)]
    pub email_alarm: EmailAlarm,
}

impl Config {
    pub fn merge(&mut self, other: &Config) {
        self.ipc.merge(&other.ipc);
        self.operating.merge(&other.operating);
        self.mining.merge(&other.mining);
        self.network.merge(&other.network);
        self.rpc.merge(&other.rpc);
        self.ws.merge(&other.ws);
        self.snapshot.merge(&other.snapshot);
        self.email_alarm.merge(&other.email_alarm);
    }

    pub fn miner_options(&self) -> Result<MinerOptions, String> {
        let (reseal_on_own_transaction, reseal_on_external_transaction) =
            match self.mining.reseal_on_txs.as_ref().map(String::as_str) {
                Some("all") => (true, true),
                Some("own") => (true, false),
                Some("ext") => (false, true),
                Some("none") => (false, false),
                Some(x) => {
                    return Err(format!(
                        "{} isn't a valid value for reseal-on-txs. Possible values are all, own, ext, none",
                        x
                    ))
                }
                None => unreachable!(),
            };

        let mem_pool_fees = MemPoolFees::create_from_options(
            self.mining.min_pay_transaction_cost,
            self.mining.min_set_regular_key_transaction_cost,
            self.mining.min_create_shard_transaction_cost,
            self.mining.min_set_shard_owners_transaction_cost,
            self.mining.min_set_shard_users_transaction_cost,
            self.mining.min_wrap_ccc_transaction_cost,
            self.mining.min_custom_transaction_cost,
            self.mining.min_store_transaction_cost,
            self.mining.min_remove_transaction_cost,
            self.mining.min_asset_mint_cost,
            self.mining.min_asset_transfer_cost,
            self.mining.min_asset_scheme_change_cost,
            self.mining.min_asset_supply_increase_cost,
            self.mining.min_asset_unwrap_ccc_cost,
        );

        Ok(MinerOptions {
            mem_pool_size: self.mining.mem_pool_size.unwrap(),
            mem_pool_memory_limit: match self.mining.mem_pool_mem_limit.unwrap() {
                0 => None,
                mem_size => Some(mem_size * 1024 * 1024),
            },
            mem_pool_fee_bump_shift: self.mining.mem_pool_fee_bump_shift.unwrap(),
            allow_create_shard: self.mining.allow_create_shard.unwrap_or(false),
            force_sealing: self.mining.force_sealing.unwrap(),
            reseal_on_own_transaction,
            reseal_on_external_transaction,
            reseal_min_period: Duration::from_millis(self.mining.reseal_min_period.unwrap()),
            reseal_max_period: Duration::from_millis(self.mining.reseal_max_period.unwrap()),
            no_reseal_timer: self.mining.no_reseal_timer.unwrap(),
            mem_pool_fees,
        })
    }

    pub fn rpc_http_config(&self) -> RpcHttpConfig {
        debug_assert!(!self.rpc.disable.unwrap());

        // FIXME: Add interface, cors and hosts options.
        RpcHttpConfig {
            interface: self.rpc.interface.clone().unwrap(),
            port: self.rpc.port.unwrap(),
            cors: self.rpc.cors.clone(),
            hosts: self.rpc.hosts.clone(),
        }
    }

    pub fn rpc_ipc_config(&self) -> RpcIpcConfig {
        debug_assert!(!self.ipc.disable.unwrap());

        RpcIpcConfig {
            socket_addr: self.ipc.path.clone().unwrap(),
        }
    }

    pub fn rpc_ws_config(&self) -> RpcWsConfig {
        debug_assert!(!self.ws.disable.unwrap());

        // FIXME: Add hosts and origins options.
        RpcWsConfig {
            interface: self.ws.interface.clone().unwrap(),
            port: self.ws.port.unwrap(),
            max_connections: self.ws.max_connections.unwrap(),
        }
    }

    pub fn network_config(&self) -> Result<NetworkConfig, String> {
        debug_assert!(!self.network.disable.unwrap());

        fn make_ipaddr_list(list_path: Option<&String>, list_name: &str) -> Result<Vec<FilterEntry>, String> {
            if let Some(path) = list_path {
                fs::read_to_string(path)
                    .map_err(|e| format!("Cannot open the {}list file {:?}: {:?}", list_name, path, e))
                    .map(|rstr| {
                        rstr.lines()
                            .map(|s| {
                                const COMMENT_CHAR: char = '#';
                                if let Some(index) = s.find(COMMENT_CHAR) {
                                    let (ip_str, tag_str_with_sign) = s.split_at(index);
                                    (ip_str.trim(), (&tag_str_with_sign[1..]).trim().to_string())
                                } else {
                                    (s.trim(), String::new())
                                }
                            })
                            .filter(|(s, _)| !s.is_empty())
                            .map(|(addr, tag)| {
                                Ok(FilterEntry {
                                    cidr: IpCidr::from_str(addr)
                                        .map_err(|e| format!("Cannot parse IP address {}: {:?}", addr, e))?,
                                    tag,
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })?
            } else {
                Ok(Vec::new())
            }
        }

        let bootstrap_addresses = self
            .network
            .bootstrap_addresses
            .clone()
            .unwrap()
            .iter()
            .map(|s| SocketAddr::from_str(s).unwrap())
            .collect::<Vec<_>>();

        let whitelist = make_ipaddr_list(self.network.whitelist_path.as_ref(), "white")?;
        let blacklist = make_ipaddr_list(self.network.blacklist_path.as_ref(), "black")?;

        Ok(NetworkConfig {
            address: self.network.interface.clone().unwrap(),
            port: self.network.port.unwrap(),
            bootstrap_addresses,
            min_peers: self.network.min_peers.unwrap(),
            max_peers: self.network.max_peers.unwrap(),
            whitelist,
            blacklist,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ipc {
    pub disable: Option<bool>,
    pub path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operating {
    pub quiet: Option<bool>,
    pub instance_id: Option<usize>,
    pub base_path: Option<String>,
    pub db_path: Option<String>,
    pub keys_path: Option<String>,
    pub password_path: Option<String>,
    pub chain: Option<ChainType>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mining {
    pub author: Option<PlatformAddress>,
    pub engine_signer: Option<PlatformAddress>,
    pub mem_pool_size: Option<usize>,
    pub mem_pool_mem_limit: Option<usize>,
    pub mem_pool_fee_bump_shift: Option<usize>,
    pub allow_create_shard: Option<bool>,
    pub force_sealing: Option<bool>,
    pub reseal_on_txs: Option<String>,
    pub reseal_min_period: Option<u64>,
    pub reseal_max_period: Option<u64>,
    pub no_reseal_timer: Option<bool>,
    pub allowed_past_gap: Option<u64>,
    pub allowed_future_gap: Option<u64>,
    pub min_pay_transaction_cost: Option<u64>,
    pub min_set_regular_key_transaction_cost: Option<u64>,
    pub min_create_shard_transaction_cost: Option<u64>,
    pub min_set_shard_owners_transaction_cost: Option<u64>,
    pub min_set_shard_users_transaction_cost: Option<u64>,
    pub min_wrap_ccc_transaction_cost: Option<u64>,
    pub min_custom_transaction_cost: Option<u64>,
    pub min_store_transaction_cost: Option<u64>,
    pub min_remove_transaction_cost: Option<u64>,
    pub min_asset_mint_cost: Option<u64>,
    pub min_asset_transfer_cost: Option<u64>,
    pub min_asset_scheme_change_cost: Option<u64>,
    pub min_asset_supply_increase_cost: Option<u64>,
    pub min_asset_unwrap_ccc_cost: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub interface: Option<String>,
    pub disable: Option<bool>,
    pub port: Option<u16>,
    pub bootstrap_addresses: Option<Vec<String>>,
    pub min_peers: Option<usize>,
    pub max_peers: Option<usize>,
    pub sync: Option<bool>,
    pub snapshot_hash: Option<H256>,
    pub snapshot_number: Option<u64>,
    pub transaction_relay: Option<bool>,
    pub discovery: Option<bool>,
    pub discovery_type: Option<String>,
    pub discovery_refresh: Option<u32>,
    pub discovery_bucket_size: Option<u8>,
    pub blacklist_path: Option<String>,
    pub whitelist_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rpc {
    pub disable: Option<bool>,
    pub interface: Option<String>,
    pub hosts: Option<Vec<String>>,
    pub cors: Option<Vec<String>>,
    pub port: Option<u16>,
    #[serde(default = "default_enable_devel_api")]
    pub enable_devel_api: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ws {
    pub disable: Option<bool>,
    pub interface: Option<String>,
    pub port: Option<u16>,
    pub max_connections: Option<usize>,
}

fn default_enable_devel_api() -> bool {
    cfg!(debug_assertions)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub disable: Option<bool>,
    pub path: Option<String>,
    // Snapshot's age in blocks
    pub expiration: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmailAlarm {
    pub disable: Option<bool>,
    pub to: Option<String>,
    pub sendgrid_key: Option<String>,
}

impl Ipc {
    pub fn merge(&mut self, other: &Ipc) {
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.path.is_some() {
            self.path = other.path.clone();
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-ipc") {
            self.disable = Some(true);
        }
        if let Some(path) = matches.value_of("ipc-path") {
            self.path = Some(path.to_string());
        }
        Ok(())
    }
}

impl Operating {
    pub fn merge(&mut self, other: &Operating) {
        if other.quiet.is_some() {
            self.quiet = other.quiet;
        }
        if other.instance_id.is_some() {
            self.instance_id = other.instance_id;
        }
        if other.base_path.is_some() {
            self.base_path = other.base_path.clone();
        }
        if other.db_path.is_some() {
            self.db_path = other.db_path.clone();
        }
        if other.keys_path.is_some() {
            self.keys_path = other.keys_path.clone();
        }
        if other.password_path.is_some() {
            self.password_path = other.password_path.clone();
        }
        if other.chain.is_some() {
            self.chain = other.chain.clone();
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("quiet") {
            self.quiet = Some(true);
        }
        if let Some(instance_id) = matches.value_of("instance-id") {
            self.instance_id = Some(instance_id.parse().map_err(|e| format!("{}", e))?);
        }
        if let Some(base_path) = matches.value_of("base-path") {
            self.base_path = Some(base_path.to_string());
        }
        if let Some(db_path) = matches.value_of("db-path") {
            self.db_path = Some(db_path.to_string());
        }
        if let Some(keys_path) = matches.value_of("keys-path") {
            self.keys_path = Some(keys_path.to_string());
        }
        if let Some(password_path) = matches.value_of("password-path") {
            self.password_path = Some(password_path.to_string());
        }
        if let Some(chain) = matches.value_of("chain") {
            self.chain = Some(chain.parse().unwrap());
        }
        Ok(())
    }
}

impl Mining {
    #[allow(clippy::cognitive_complexity)]
    pub fn merge(&mut self, other: &Mining) {
        if other.author.is_some() {
            self.author = other.author;
        }
        if other.engine_signer.is_some() {
            self.engine_signer = other.engine_signer;
        }
        if other.mem_pool_size.is_some() {
            self.mem_pool_size = other.mem_pool_size;
        }
        if other.mem_pool_mem_limit.is_some() {
            self.mem_pool_mem_limit = other.mem_pool_mem_limit;
        }
        if other.allow_create_shard.is_some() {
            self.allow_create_shard = other.allow_create_shard;
        }
        if other.force_sealing.is_some() {
            self.force_sealing = other.force_sealing;
        }
        if other.reseal_on_txs.is_some() {
            self.reseal_on_txs = other.reseal_on_txs.clone();
        }
        if other.reseal_min_period.is_some() {
            self.reseal_min_period = other.reseal_min_period;
        }
        if other.reseal_max_period.is_some() {
            self.reseal_max_period = other.reseal_max_period;
        }
        if other.no_reseal_timer.is_some() {
            self.no_reseal_timer = other.no_reseal_timer;
        }
        if other.min_pay_transaction_cost.is_some() {
            self.min_pay_transaction_cost = other.min_pay_transaction_cost;
        }
        if other.min_set_regular_key_transaction_cost.is_some() {
            self.min_set_regular_key_transaction_cost = other.min_set_regular_key_transaction_cost;
        }
        if other.min_create_shard_transaction_cost.is_some() {
            self.min_create_shard_transaction_cost = other.min_create_shard_transaction_cost;
        }
        if other.min_set_shard_owners_transaction_cost.is_some() {
            self.min_set_shard_owners_transaction_cost = other.min_set_shard_owners_transaction_cost;
        }
        if other.min_set_shard_users_transaction_cost.is_some() {
            self.min_set_shard_users_transaction_cost = other.min_set_shard_users_transaction_cost;
        }
        if other.min_wrap_ccc_transaction_cost.is_some() {
            self.min_wrap_ccc_transaction_cost = other.min_wrap_ccc_transaction_cost;
        }
        if other.min_custom_transaction_cost.is_some() {
            self.min_custom_transaction_cost = other.min_custom_transaction_cost;
        }
        if other.min_store_transaction_cost.is_some() {
            self.min_store_transaction_cost = other.min_store_transaction_cost;
        }
        if other.min_remove_transaction_cost.is_some() {
            self.min_remove_transaction_cost = other.min_remove_transaction_cost;
        }
        if other.min_asset_mint_cost.is_some() {
            self.min_asset_mint_cost = other.min_asset_mint_cost;
        }
        if other.min_asset_transfer_cost.is_some() {
            self.min_asset_transfer_cost = other.min_asset_transfer_cost;
        }
        if other.min_asset_scheme_change_cost.is_some() {
            self.min_asset_scheme_change_cost = other.min_asset_scheme_change_cost;
        }
        if other.min_asset_supply_increase_cost.is_some() {
            self.min_asset_supply_increase_cost = other.min_asset_supply_increase_cost;
        }
        if other.min_asset_unwrap_ccc_cost.is_some() {
            self.min_asset_unwrap_ccc_cost = other.min_asset_unwrap_ccc_cost;
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if let Some(author) = matches.value_of("author") {
            self.author = Some(author.parse().map_err(|_| "Invalid address format")?);
        }
        if let Some(engine_signer) = matches.value_of("engine-signer") {
            self.engine_signer = Some(engine_signer.parse().map_err(|_| "Invalid address format")?);
        }
        if matches.is_present("no-miner") {
            self.author = None;
            self.engine_signer = None;
            println!("This option was deprecated. PBFT or PoA type engine with no author implicitly means no-miner.");
        }
        if let Some(mem_pool_fee_bump_shift) = matches.value_of("mem-pool-fee-bump-shift") {
            self.mem_pool_mem_limit =
                Some(mem_pool_fee_bump_shift.parse().map_err(|_| "Invalid mem pool fee bump shift")?);
        }
        if let Some(mem_pool_mem_limit) = matches.value_of("mem-pool-mem-limit") {
            self.mem_pool_mem_limit = Some(mem_pool_mem_limit.parse().map_err(|_| "Invalid mem limit")?);
        }
        if let Some(mem_pool_size) = matches.value_of("mem-pool-size") {
            self.mem_pool_size = Some(mem_pool_size.parse().map_err(|_| "Invalid size")?);
        }
        if matches.is_present("allow-create-shard") {
            self.allow_create_shard = Some(true)
        }
        if matches.is_present("force-sealing") {
            self.force_sealing = Some(true);
        }
        if let Some(reseal_on_txs) = matches.value_of("reseal-on-txs") {
            self.reseal_on_txs = Some(reseal_on_txs.to_string());
        }
        if let Some(reseal_min_period) = matches.value_of("reseal-min-period") {
            self.reseal_min_period = Some(reseal_min_period.parse().map_err(|_| "Invalid period")?);
        }
        if let Some(reseal_max_period) = matches.value_of("reseal-max-period") {
            self.reseal_max_period = Some(reseal_max_period.parse().map_err(|_| "Invalid period")?);
        }
        if matches.is_present("no-reseal-timer") {
            self.no_reseal_timer = Some(true);
        }
        if let Some(allowed_past_gap) = matches.value_of("allowed-past-gap") {
            self.allowed_past_gap = Some(allowed_past_gap.parse().map_err(|_| "Invalid time gap")?);
        }
        if let Some(allowed_future_gap) = matches.value_of("allowed-future-gap") {
            self.allowed_future_gap = Some(allowed_future_gap.parse().map_err(|_| "Invalid time gap")?);
        }
        Ok(())
    }

    pub fn create_time_gaps(&self) -> TimeGapParams {
        let allowed_past_gap = Duration::from_millis(self.allowed_past_gap.unwrap_or(30000));
        let allowed_future_gap = Duration::from_millis(self.allowed_future_gap.unwrap_or(5000));

        TimeGapParams {
            allowed_past_gap,
            allowed_future_gap,
        }
    }
}

impl Network {
    pub fn merge(&mut self, other: &Network) {
        if other.interface.is_some() {
            self.interface = other.interface.clone();
        }
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.port.is_some() {
            self.port = other.port;
        }
        if other.bootstrap_addresses.is_some() {
            self.bootstrap_addresses = other.bootstrap_addresses.clone();
        }
        if other.min_peers.is_some() {
            self.min_peers = other.min_peers;
        }
        if other.max_peers.is_some() {
            self.max_peers = other.max_peers;
        }
        if other.sync.is_some() {
            self.sync = other.sync;
        }
        if other.snapshot_hash.is_some() {
            self.snapshot_hash = other.snapshot_hash;
        }
        if other.snapshot_number.is_some() {
            self.snapshot_number = other.snapshot_number;
        }
        if other.transaction_relay.is_some() {
            self.transaction_relay = other.transaction_relay;
        }
        if other.discovery.is_some() {
            self.discovery = other.discovery;
        }
        if other.discovery_type.is_some() {
            self.discovery_type = other.discovery_type.clone();
        }
        if other.discovery_refresh.is_some() {
            self.discovery_refresh = other.discovery_refresh;
        }
        if other.discovery_bucket_size.is_some() {
            self.discovery_bucket_size = other.discovery_bucket_size;
        }
        if other.blacklist_path.is_some() {
            self.blacklist_path = other.blacklist_path.clone();
        }
        if other.whitelist_path.is_some() {
            self.whitelist_path = other.whitelist_path.clone();
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-network") {
            self.disable = Some(true);
        }

        if let Some(addresses) = matches.values_of("bootstrap-addresses") {
            self.bootstrap_addresses = Some(addresses.map(|a| a.into()).collect());
        }

        if let Some(interface) = matches.value_of("interface") {
            self.interface = Some(interface.to_string());
        }
        if let Some(port) = matches.value_of("port") {
            self.port = Some(port.parse().map_err(|_| "Invalid port")?);
        }

        if let Some(min_peers) = matches.value_of("min-peers") {
            self.min_peers = Some(min_peers.parse().map_err(|_| "Invalid min-peers")?);
        }
        if let Some(max_peers) = matches.value_of("max-peers") {
            self.max_peers = Some(max_peers.parse().map_err(|_| "Invalid max-peers")?);
        }
        if self.min_peers > self.max_peers {
            return Err("Invalid min/max peers".to_string())
        }

        if matches.is_present("no-sync") {
            self.sync = Some(false);
        }
        if let Some(snapshot_hash) = matches.value_of("snapshot-hash") {
            self.snapshot_hash = Some(snapshot_hash.parse().map_err(|_| "Invalid snapshot-hash")?);
        }
        if let Some(snapshot_number) = matches.value_of("snapshot-number") {
            self.snapshot_number = Some(snapshot_number.parse().map_err(|_| "Invalid snapshot-number")?);
        }
        if matches.is_present("no-tx-relay") {
            self.transaction_relay = Some(false);
        }

        if matches.is_present("no-discovery") {
            self.discovery = Some(false);
        }
        if let Some(discovery_type) = matches.value_of("discovery") {
            self.discovery_type = Some(discovery_type.to_string());
        }
        if let Some(refresh) = matches.value_of("discovery-refresh") {
            self.discovery_refresh = Some(refresh.parse().map_err(|_| "Invalid discovery-refresh")?);
        }
        if let Some(bucket_size) = matches.value_of("discovery-bucket-size") {
            self.discovery_bucket_size = Some(bucket_size.parse().map_err(|_| "Invalid discovery-bucket-size")?);
        }

        if let Some(file_path) = matches.value_of("whitelist-path") {
            self.whitelist_path = Some(file_path.to_string());
        }
        if let Some(file_path) = matches.value_of("blacklist-path") {
            self.blacklist_path = Some(file_path.to_string());
        }

        Ok(())
    }
}

impl Rpc {
    pub fn merge(&mut self, other: &Rpc) {
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.interface.is_some() {
            self.interface = other.interface.clone();
        }
        if other.hosts.is_some() {
            self.hosts = other.hosts.clone();
        }
        if other.cors.is_some() {
            self.cors = other.cors.clone();
        }
        if other.port.is_some() {
            self.port = other.port;
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-jsonrpc") {
            self.disable = Some(true);
        }
        if let Some(port) = matches.value_of("jsonrpc-port") {
            self.port = Some(port.parse().map_err(|_| "Invalid port")?);
        }
        if let Some(interface) = matches.value_of("jsonrpc-interface") {
            self.interface = Some(interface.to_string());
        }
        if let Some(hosts) = matches.values_of_lossy("jsonrpc-hosts") {
            self.hosts = Some(hosts);
        }
        if let Some(cors) = matches.values_of_lossy("jsonrpc-cors") {
            self.cors = Some(cors);
        }
        if matches.is_present("enable-devel-api") {
            self.enable_devel_api = true;
        }
        Ok(())
    }
}

impl Ws {
    pub fn merge(&mut self, other: &Ws) {
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.interface.is_some() {
            self.interface = other.interface.clone();
        }
        if other.port.is_some() {
            self.port = other.port;
        }
        if other.max_connections.is_some() {
            self.max_connections = other.max_connections;
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-ws") {
            self.disable = Some(true);
        }

        if let Some(interface) = matches.value_of("ws-interface") {
            self.interface = Some(interface.to_string());
        }
        if let Some(port) = matches.value_of("ws-port") {
            self.port = Some(port.parse().map_err(|_| "Invalid port")?);
        }
        if let Some(max_connections) = matches.value_of("ws-max-connections") {
            self.max_connections = Some(max_connections.parse().map_err(|_| "Invalid max connections")?);
        }
        Ok(())
    }
}

impl Snapshot {
    pub fn merge(&mut self, other: &Snapshot) {
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.path.is_some() {
            self.path = other.path.clone();
        }
        if other.expiration.is_some() {
            self.expiration = other.expiration;
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-snapshot") {
            self.disable = Some(true);
        }

        if let Some(snapshot_path) = matches.value_of("snapshot-path") {
            self.path = Some(snapshot_path.to_string());
        }
        Ok(())
    }
}

impl EmailAlarm {
    pub fn merge(&mut self, other: &EmailAlarm) {
        if other.disable.is_some() {
            self.disable = other.disable;
        }
        if other.to.is_some() {
            self.to = other.to.clone();
        }
        if other.sendgrid_key.is_some() {
            self.sendgrid_key = other.sendgrid_key.clone();
        }
    }

    pub fn overwrite_with(&mut self, matches: &clap::ArgMatches<'_>) -> Result<(), String> {
        if matches.is_present("no-email-alarm") {
            self.disable = Some(true);
        }
        if let Some(to) = matches.value_of("email-alarm-to") {
            self.to = Some(to.to_string());
        }
        if let Some(sendgrid_key) = matches.value_of("email-alarm-sendgrid-key") {
            self.sendgrid_key = Some(sendgrid_key.to_string());
        }

        Ok(())
    }
}

impl Default for EmailAlarm {
    fn default() -> Self {
        Self {
            disable: Some(true),
            to: None,
            sendgrid_key: None,
        }
    }
}

#[cfg(not(debug_assertions))]
pub fn read_preset_config() -> &'static str {
    let bytes = include_bytes!("presets/config.prod.toml");
    str::from_utf8(bytes).expect("The preset config file must be valid")
}

#[cfg(debug_assertions)]
pub fn read_preset_config() -> &'static str {
    let bytes = include_bytes!("presets/config.dev.toml");
    str::from_utf8(bytes).expect("The preset config file must be valid")
}

pub fn load_config(matches: &clap::ArgMatches<'_>) -> Result<Config, String> {
    let mut config: Config = {
        let toml_string = read_preset_config().to_string();
        toml::from_str(toml_string.as_ref()).expect("The preset config file must be valid")
    };

    if let Some(config_path) = matches.value_of("config") {
        let toml_string = fs::read_to_string(config_path).map_err(|e| format!("Fail to read file: {:?}", e))?;
        let extra_config: Config =
            toml::from_str(toml_string.as_ref()).map_err(|e| format!("Error while parsing TOML: {:?}", e))?;
        config.merge(&extra_config);
    };

    config.ipc.overwrite_with(&matches)?;
    config.operating.overwrite_with(&matches)?;
    config.mining.overwrite_with(&matches)?;
    config.network.overwrite_with(&matches)?;
    config.rpc.overwrite_with(&matches)?;
    config.ws.overwrite_with(&matches)?;
    config.snapshot.overwrite_with(&matches)?;
    config.email_alarm.overwrite_with(&matches)?;
    Ok(config)
}
