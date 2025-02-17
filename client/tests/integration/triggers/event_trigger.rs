use std::str::FromStr;

use eyre::Result;
use iroha_client::{
    client::{self, Client},
    data_model::prelude::*,
};
use test_network::*;

#[test]
fn test_mint_asset_when_new_asset_definition_created() -> Result<()> {
    let (_rt, _peer, mut test_client) = <PeerBuilder>::new().with_port(10_770).start_with_runtime();
    wait_for_genesis_committed(&vec![test_client.clone()], 0);

    let asset_definition_id = "rose#wonderland".parse()?;
    let account_id = AccountId::from_str("alice@wonderland")?;
    let asset_id = AssetId::new(asset_definition_id, account_id.clone());
    let prev_value = get_asset_value(&mut test_client, asset_id.clone())?;

    let instruction = Mint::asset_quantity(1_u32, asset_id.clone());
    let register_trigger = Register::trigger(Trigger::new(
        "mint_rose".parse()?,
        Action::new(
            vec![instruction],
            Repeats::Indefinitely,
            account_id,
            // FIXME: rewrite the filters using the builder DSL https://github.com/hyperledger/iroha/issues/3068
            TriggeringFilterBox::Data(BySome(DataEntityFilter::ByDomain(BySome(
                DomainFilter::new(
                    AcceptAll,
                    BySome(DomainEventFilter::ByAssetDefinition(BySome(
                        AssetDefinitionFilter::new(
                            AcceptAll,
                            BySome(AssetDefinitionEventFilter::ByCreated),
                        ),
                    ))),
                ),
            )))),
        ),
    ));
    test_client.submit(register_trigger)?;

    let tea_definition_id = "tea#wonderland".parse()?;
    let register_tea_definition =
        Register::asset_definition(AssetDefinition::quantity(tea_definition_id));
    test_client.submit_blocking(register_tea_definition)?;

    let new_value = get_asset_value(&mut test_client, asset_id)?;
    assert_eq!(new_value, prev_value + 1);

    Ok(())
}

fn get_asset_value(client: &mut Client, asset_id: AssetId) -> Result<u32> {
    let asset = client.request(client::asset::by_id(asset_id))?;
    Ok(*TryAsRef::<u32>::try_as_ref(asset.value())?)
}
