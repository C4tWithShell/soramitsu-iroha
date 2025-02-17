import allure
import pytest

from common.consts import Stderr
from src.client_cli import client_cli, have, iroha

@pytest.fixture(scope="function", autouse=True)
def story_account_registers_asset_definitions():
    allure.dynamic.story('Account registers an asset definition')
    allure.dynamic.label('permission', 'no_permission_required')


@allure.label('sdk_test_id', 'register_asset_definition_with_quantity_value_type')
def test_register_asset_definition_with_quantity_value_type(
        GIVEN_fake_asset_name,
        GIVEN_new_one_existing_domain,
        GIVEN_quantity_value_type):
    with allure.step(
            f'WHEN client_cli registers the asset_definition "{GIVEN_fake_asset_name}" '
            f'with "{GIVEN_quantity_value_type}" value type'
            f'in the "{GIVEN_new_one_existing_domain.name}" domain'):
        client_cli.register().asset().definition(
            asset=GIVEN_fake_asset_name,
            domain=GIVEN_new_one_existing_domain.name,
            value_type=GIVEN_quantity_value_type)
    with allure.step(
            f'THEN Iroha should have the asset "{GIVEN_fake_asset_name}"'):
        iroha.should(have.asset_definition(GIVEN_fake_asset_name + '#' + GIVEN_new_one_existing_domain.name))

@allure.label('sdk_test_id', 'register_asset_definition_with_store_value_type')
def test_register_asset_definition_with_store_value_type(
        GIVEN_fake_asset_name,
        GIVEN_new_one_existing_domain,
        GIVEN_store_value_type):
    with allure.step(
            f'WHEN client_cli registers the asset_definition "{GIVEN_fake_asset_name}" '
            f'with "{GIVEN_store_value_type}" value type'
            f'in the "{GIVEN_new_one_existing_domain.name}" domain'):
        client_cli.register().asset().definition(
            asset=GIVEN_fake_asset_name,
            domain=GIVEN_new_one_existing_domain.name,
            value_type=GIVEN_store_value_type)
    with allure.step(
            f'THEN Iroha should have the asset "{GIVEN_fake_asset_name}"'):
        iroha.should(have.asset_definition(GIVEN_fake_asset_name + '#' + GIVEN_new_one_existing_domain.name))

@allure.label("sdk_test_id", "register_asset_definition_with_metadata")
@pytest.mark.xfail(reason="TO DO")
def test_register_asset_definition_with_metadata(
        GIVEN_fake_asset_name,
        GIVEN_new_one_existing_domain):
    assert 0


@allure.label("sdk_test_id", "register_fixed_asset_definition")
@pytest.mark.xfail(reason="TO DO")
def test_register_fixed_asset_definition(
        GIVEN_fake_asset_name,
        GIVEN_new_one_existing_domain):
    assert 0


@allure.label('sdk_test_id', 'register_asset_with_existing_name')
def test_register_asset_with_existing_name(
        GIVEN_existing_asset_definition_with_quantity_value_type):
    with allure.step(
            f'WHEN client_cli tries to register an asset definition with the same name "{GIVEN_existing_asset_definition_with_quantity_value_type.name}"'
            f'in the "{GIVEN_existing_asset_definition_with_quantity_value_type.domain}" domain'):
        client_cli.register().asset().definition(asset=GIVEN_existing_asset_definition_with_quantity_value_type.name,
                                                 domain=GIVEN_existing_asset_definition_with_quantity_value_type.domain,
                                                 value_type=GIVEN_existing_asset_definition_with_quantity_value_type.value_type)
    with allure.step(
            f'THEN client_cli should have the asset definition error: "{GIVEN_existing_asset_definition_with_quantity_value_type.__repr__()}"'):
        client_cli.should(have.error(Stderr.REPETITION.value))



@allure.label('sdk_test_id', 'register_asset_with_empty_name')
def test_register_asset_with_empty_name(
        GIVEN_new_one_existing_domain):
    with allure.step(
            f'WHEN client_cli tries to register an asset definition with an empty name'
            f'in the "{GIVEN_new_one_existing_domain.name}" domain'):
        client_cli.register().asset().definition(asset='', domain=GIVEN_new_one_existing_domain.name, value_type='Quantity')
    with allure.step(
            f'THEN сlient_cli should have the asset error: "{Stderr.CANNOT_BE_EMPTY}"'):
        client_cli.should(have.error(Stderr.CANNOT_BE_EMPTY.value))



@allure.label('sdk_test_id', 'register_asset_with_not_existing_domain')
def test_register_asset_with_not_existing_domain(
        GIVEN_not_existing_name,
        GIVEN_quantity_value_type,
        GIVEN_fake_asset_name):
    with allure.step(
            f'WHEN client_cli tries to register an asset definition with not existing domain'):
        client_cli.register().asset().definition(asset=GIVEN_fake_asset_name, domain=GIVEN_not_existing_name,
                                                 value_type=GIVEN_quantity_value_type)
    with allure.step(
            f'THEN client_cli should have the error'):
        client_cli.should(have.error(Stderr.FAILED_TO_FIND_DOMAIN.value))



@allure.label('sdk_test_id', 'register_asset_with_too_long_value_type')
def test_register_asset_with_too_long_value_type(
        GIVEN_fake_asset_name,
        GIVEN_new_one_existing_domain):
    with allure.step(
            'WHEN client_cli tries to register an asset definition with too long value type'):
        client_cli.register().asset().definition(asset=GIVEN_fake_asset_name,
                                                 domain=GIVEN_new_one_existing_domain.name,
                                                 value_type='coin')
    with allure.step(
            'THEN client_cli should have the error'):
        client_cli.should(have.error(Stderr.INVALID_VALUE_TYPE.value))
