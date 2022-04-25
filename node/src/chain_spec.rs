use hex_literal::hex;
use node_april_dao_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

type Balance = u128;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	let pre_funded_account: Vec<(AccountId, Balance)> = vec![
		(
			hex!("30a056196515173d8d5adeb538d74d74d4dcffd992347bffcfa8bbee3f285a75").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("8434b2f2d8194d44455d9e37869cf515a38aaed473f01c8bfccb545ed926fa0c").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("b2c5459a18340237a8215c8314c9ae0c10a271a8b01fc561310bcfebb5beb75d").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("a40ea9e24270a9bd77d9124435b335da380dda55cfc7343ec850318beaefb965").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("8629035adee2e7f4d01009fc7239ab86a069508d829d833947fb3d26d6059645").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("aa1cd0c52630c22b7a3d49e3627c0430c5c1af134973e35e5a36d594b36ef406").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("18083daa4eb9720c467c944767c8e2d600a977a40306a45f449ffa24b9fc3164").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("e2ee03a533df59dd4333f70c809b8952634b1171b4325ecdd0da9c1df6e9482a").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("16a1b71a9dd4613ad4e79e526e508290ffbfcf275f14f6e8e69635d72cb2ed78").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("04544defb7c9825b7795fd3e825199b8b3b4edaec3f0337a416c8838aad46906").into(),
			100_000_000_000_000_000,
		),
		(
			hex!("fe0eb36e00c8dce69d8d274646b5e9114931d944f2c6c6f3d0590746a6760943").into(),
			100_000_000_000_000_000,
		),
		(get_account_id_from_seed::<sr25519::Public>("Alice"), 100_000_000_000_000_000),
		(get_account_id_from_seed::<sr25519::Public>("Bob"), 100_000_000_000_000_000),
		(get_account_id_from_seed::<sr25519::Public>("Charlie"), 100_000_000_000_000_000),
		(get_account_id_from_seed::<sr25519::Public>("Dave"), 100_000_000_000_000_000),
		(get_account_id_from_seed::<sr25519::Public>("Eve"), 100_000_000_000_000_000),
		(get_account_id_from_seed::<sr25519::Public>("Ferdie"), 100_000_000_000_000_000),
	];

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			dev_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				pre_funded_account.clone(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		assets: Default::default(),
	}
}

fn dev_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, Balance)>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k.0, k.1)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		assets: Default::default(),
	}
}
