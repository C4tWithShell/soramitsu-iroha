use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};
use iroha_config::{sumeragi::default::*, wasm::default::*, wsv::default::*};
use iroha_data_model::{
    asset::AssetValueType,
    metadata::Limits,
    parameter::{default::*, ParametersBuilder},
    prelude::AssetId,
};
use iroha_genesis::{ExecutorMode, ExecutorPath, RawGenesisBlock, RawGenesisBlockBuilder};
use serde_json::json;

use super::*;

const INLINED_EXECUTOR_WARNING: &str = r"WARN: You're using genesis with inlined executor.
Consider specifying a separate executor file using `--executor-path-in-genesis` instead.
Use `--help` for more information.";

#[derive(Parser, Debug, Clone)]
#[clap(group = ArgGroup::new("executor").required(true))]
pub struct Args {
    /// Reads the executor from the file at <PATH> (relative to CWD)
    /// and includes the content into the genesis.
    ///
    /// WARN: This approach can lead to reproducibility issues, as WASM builds are currently not
    /// guaranteed to be reproducible. Additionally, inlining the executor bloats the genesis JSON
    /// and makes it less readable. Consider specifying a separate executor file
    /// using `--executor-path-in-genesis` instead. For more details, refer to
    /// the related PR: https://github.com/hyperledger/iroha/pull/3434
    #[clap(long, group = "executor", value_name = "PATH")]
    inline_executor_from_file: Option<PathBuf>,
    /// Specifies the <PATH> that will be directly inserted into the genesis JSON as-is.
    #[clap(long, group = "executor", value_name = "PATH")]
    executor_path_in_genesis: Option<PathBuf>,
    #[clap(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand, Debug, Clone, Default)]
pub enum Mode {
    /// Generate default genesis
    #[default]
    Default,
    /// Generate synthetic genesis with specified number of domains, accounts and assets.
    ///
    /// Synthetic mode is useful when we need a semi-realistic genesis for stress-testing
    /// Iroha's startup times as well as being able to just start an Iroha network and have
    /// instructions that represent a typical blockchain after migration.
    Synthetic {
        /// Number of domains in synthetic genesis.
        #[clap(long, default_value_t)]
        domains: u64,
        /// Number of accounts per domains in synthetic genesis.
        /// Total number of  accounts would be `domains * assets_per_domain`.
        #[clap(long, default_value_t)]
        accounts_per_domain: u64,
        /// Number of assets per domains in synthetic genesis.
        /// Total number of assets would be `domains * assets_per_domain`.
        #[clap(long, default_value_t)]
        assets_per_domain: u64,
    },
}

impl<T: Write> RunArgs<T> for Args {
    fn run(self, writer: &mut BufWriter<T>) -> Outcome {
        let Self {
            inline_executor_from_file,
            executor_path_in_genesis,
            mode,
        } = self;

        let executor: ExecutorMode = match (inline_executor_from_file, executor_path_in_genesis) {
            (Some(path), None) => {
                eprintln!("{INLINED_EXECUTOR_WARNING}");
                ParsedExecutorArgs::Inline(path)
            }
            (None, Some(path)) => ParsedExecutorArgs::Path(path),
            _ => unreachable!("clap invariant"),
        }
        .try_into()?;

        let genesis = match mode.unwrap_or_default() {
            Mode::Default => generate_default(executor),
            Mode::Synthetic {
                domains,
                accounts_per_domain,
                assets_per_domain,
            } => generate_synthetic(executor, domains, accounts_per_domain, assets_per_domain),
        }?;
        writeln!(writer, "{}", serde_json::to_string_pretty(&genesis)?)
            .wrap_err("Failed to write serialized genesis to the buffer.")
    }
}

enum ParsedExecutorArgs {
    Inline(PathBuf),
    Path(PathBuf),
}

impl TryFrom<ParsedExecutorArgs> for ExecutorMode {
    type Error = color_eyre::Report;

    fn try_from(value: ParsedExecutorArgs) -> Result<Self, Self::Error> {
        let mode = match value {
            ParsedExecutorArgs::Path(path) => ExecutorMode::Path(ExecutorPath(path)),
            ParsedExecutorArgs::Inline(path) => {
                let executor = ExecutorMode::Path(ExecutorPath(path.clone()))
                    .try_into()
                    .wrap_err_with(|| {
                        format!("Failed to read the executor located at {}", path.display())
                    })?;
                ExecutorMode::Inline(executor)
            }
        };
        Ok(mode)
    }
}

#[allow(clippy::too_many_lines)]
pub fn generate_default(executor: ExecutorMode) -> color_eyre::Result<RawGenesisBlock> {
    let mut meta = Metadata::new();
    meta.insert_with_limits(
        "key".parse()?,
        "value".to_owned().into(),
        Limits::new(1024, 1024),
    )?;

    let mut genesis = RawGenesisBlockBuilder::default()
        .domain_with_metadata("wonderland".parse()?, meta.clone())
        .account_with_metadata(
            "alice".parse()?,
            crate::DEFAULT_PUBLIC_KEY.parse()?,
            meta.clone(),
        )
        .account_with_metadata("bob".parse()?, crate::DEFAULT_PUBLIC_KEY.parse()?, meta)
        .asset("rose".parse()?, AssetValueType::Quantity)
        .finish_domain()
        .domain("garden_of_live_flowers".parse()?)
        .account("carpenter".parse()?, crate::DEFAULT_PUBLIC_KEY.parse()?)
        .asset("cabbage".parse()?, AssetValueType::Quantity)
        .finish_domain()
        .executor(executor)
        .build();

    let alice_id = AccountId::from_str("alice@wonderland")?;
    let mint = Mint::asset_quantity(
        13_u32,
        AssetId::new("rose#wonderland".parse()?, alice_id.clone()),
    );
    let mint_cabbage = Mint::asset_quantity(
        44_u32,
        AssetId::new("cabbage#garden_of_live_flowers".parse()?, alice_id.clone()),
    );
    let grant_permission_to_set_parameters = Grant::permission(
        PermissionToken::new("CanSetParameters".parse()?, &json!(null)),
        alice_id.clone(),
    );
    let transfer_domain_ownerhip = Transfer::domain(
        "genesis@genesis".parse()?,
        "wonderland".parse()?,
        alice_id.clone(),
    );
    let register_user_metadata_access = Register::role(
        Role::new("ALICE_METADATA_ACCESS".parse()?)
            .add_permission(PermissionToken::new(
                "CanSetKeyValueInUserAccount".parse()?,
                &json!({ "account_id": alice_id }),
            ))
            .add_permission(PermissionToken::new(
                "CanRemoveKeyValueInUserAccount".parse()?,
                &json!({ "account_id": alice_id }),
            )),
    )
    .into();

    let parameter_defaults = ParametersBuilder::new()
        .add_parameter(MAX_TRANSACTIONS_IN_BLOCK, DEFAULT_MAX_TRANSACTIONS_IN_BLOCK)?
        .add_parameter(BLOCK_TIME, DEFAULT_BLOCK_TIME_MS)?
        .add_parameter(COMMIT_TIME_LIMIT, DEFAULT_COMMIT_TIME_LIMIT_MS)?
        .add_parameter(TRANSACTION_LIMITS, DEFAULT_TRANSACTION_LIMITS)?
        .add_parameter(WSV_ASSET_METADATA_LIMITS, DEFAULT_METADATA_LIMITS)?
        .add_parameter(
            WSV_ASSET_DEFINITION_METADATA_LIMITS,
            DEFAULT_METADATA_LIMITS.to_value(),
        )?
        .add_parameter(WSV_ACCOUNT_METADATA_LIMITS, DEFAULT_METADATA_LIMITS)?
        .add_parameter(WSV_DOMAIN_METADATA_LIMITS, DEFAULT_METADATA_LIMITS)?
        .add_parameter(WSV_IDENT_LENGTH_LIMITS, DEFAULT_IDENT_LENGTH_LIMITS)?
        .add_parameter(WASM_FUEL_LIMIT, DEFAULT_FUEL_LIMIT)?
        .add_parameter(WASM_MAX_MEMORY, DEFAULT_MAX_MEMORY)?
        .into_create_parameters();

    let first_tx = genesis
        .first_transaction_mut()
        .expect("At least one transaction is expected");
    for isi in [
        mint.into(),
        mint_cabbage.into(),
        transfer_domain_ownerhip.into(),
        grant_permission_to_set_parameters.into(),
    ]
    .into_iter()
    .chain(parameter_defaults.into_iter())
    .chain(std::iter::once(register_user_metadata_access))
    {
        first_tx.append_instruction(isi);
    }

    Ok(genesis)
}

fn generate_synthetic(
    executor: ExecutorMode,
    domains: u64,
    accounts_per_domain: u64,
    assets_per_domain: u64,
) -> color_eyre::Result<RawGenesisBlock> {
    // Add default `Domain` and `Account` to still be able to query
    let mut builder = RawGenesisBlockBuilder::default()
        .domain("wonderland".parse()?)
        .account("alice".parse()?, crate::DEFAULT_PUBLIC_KEY.parse()?)
        .finish_domain();

    for domain in 0..domains {
        let mut domain_builder = builder.domain(format!("domain_{domain}").parse()?);

        for account in 0..accounts_per_domain {
            let (public_key, _) = iroha_crypto::KeyPair::generate()?.into();
            domain_builder =
                domain_builder.account(format!("account_{account}").parse()?, public_key);
        }

        for asset in 0..assets_per_domain {
            domain_builder =
                domain_builder.asset(format!("asset_{asset}").parse()?, AssetValueType::Quantity);
        }

        builder = domain_builder.finish_domain();
    }
    let mut genesis = builder.executor(executor).build();

    let first_transaction = genesis
        .first_transaction_mut()
        .expect("At least one transaction is expected");
    for domain in 0..domains {
        for account in 0..accounts_per_domain {
            // FIXME: it actually generates (assets_per_domain * accounts_per_domain) assets per domain
            //        https://github.com/hyperledger/iroha/issues/3508
            for asset in 0..assets_per_domain {
                let mint = Mint::asset_quantity(
                    13_u32,
                    AssetId::new(
                        format!("asset_{asset}#domain_{domain}").parse()?,
                        format!("account_{account}@domain_{domain}").parse()?,
                    ),
                )
                .into();
                first_transaction.append_instruction(mint);
            }
        }
    }

    Ok(genesis)
}
