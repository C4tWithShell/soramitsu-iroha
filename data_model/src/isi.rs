//! This library contains basic Iroha Special Instructions.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
use core::fmt::{Debug, Display};

use derive_more::{Constructor, DebugCustom, Display};
use iroha_data_model_derive::model;
use iroha_schema::IntoSchema;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

pub use self::{model::*, transparent::*};
use super::{prelude::*, Value};
use crate::{seal, Level, Registered};

/// Marker trait designating instruction.
///
/// Instructions allows to change the state of `Iroha`.
/// All possible instructions are implementors of this trait, excluding
/// [`InstructionBox`] which is just a wrapper.
pub trait Instruction: Into<InstructionBox> + seal::Sealed {}

#[model]
pub mod model {
    pub use transparent::*;

    use super::*;

    /// Sized structure for all possible Instructions.
    ///
    /// Note that [`InstructionBox`] is not a self-sufficient instruction,
    /// but just a wrapper to pass instructions back and forth.
    /// If you are a client SDK user then you likely don't need to use this type directly.
    #[derive(
        DebugCustom,
        Display,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        EnumDiscriminants,
        Decode,
        Encode,
        Deserialize,
        Serialize,
        IntoSchema,
    )]
    #[strum_discriminants(
        name(InstructionType),
        derive(
            Display,
            PartialOrd,
            Ord,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema
        ),
        cfg_attr(
            any(feature = "ffi_import", feature = "ffi_export"),
            derive(iroha_ffi::FfiType)
        ),
        allow(missing_docs),
        repr(u8)
    )]
    #[ffi_type(opaque)]
    #[allow(missing_docs)]
    pub enum InstructionBox {
        #[debug(fmt = "{_0:?}")]
        Register(RegisterBox),
        #[debug(fmt = "{_0:?}")]
        Unregister(UnregisterBox),
        #[debug(fmt = "{_0:?}")]
        Mint(MintBox),
        #[debug(fmt = "{_0:?}")]
        Burn(BurnBox),
        #[debug(fmt = "{_0:?}")]
        Transfer(TransferBox),
        #[debug(fmt = "{_0:?}")]
        SetKeyValue(SetKeyValueBox),
        #[debug(fmt = "{_0:?}")]
        RemoveKeyValue(RemoveKeyValueBox),
        #[debug(fmt = "{_0:?}")]
        Grant(GrantBox),
        #[debug(fmt = "{_0:?}")]
        Revoke(RevokeBox),
        #[debug(fmt = "{_0:?}")]
        ExecuteTrigger(ExecuteTrigger),
        #[debug(fmt = "{_0:?}")]
        SetParameter(SetParameter),
        #[debug(fmt = "{_0:?}")]
        NewParameter(NewParameter),
        #[debug(fmt = "{_0:?}")]
        Upgrade(Upgrade),
        #[debug(fmt = "{_0:?}")]
        Log(Log),

        #[debug(fmt = "{_0:?}")]
        Fail(Fail),
    }

    impl Instruction for InstructionBox {}

    impl Instruction for SetKeyValue<Domain> {}
    impl Instruction for SetKeyValue<Account> {}
    impl Instruction for SetKeyValue<AssetDefinition> {}
    impl Instruction for SetKeyValue<Asset> {}

    impl Instruction for RemoveKeyValue<Domain> {}
    impl Instruction for RemoveKeyValue<Account> {}
    impl Instruction for RemoveKeyValue<AssetDefinition> {}
    impl Instruction for RemoveKeyValue<Asset> {}

    impl Instruction for Register<Peer> {}
    impl Instruction for Register<Domain> {}
    impl Instruction for Register<Account> {}
    impl Instruction for Register<AssetDefinition> {}
    impl Instruction for Register<Asset> {}
    impl Instruction for Register<Role> {}
    impl Instruction for Register<Trigger<TriggeringFilterBox>> {}

    impl Instruction for Unregister<Peer> {}
    impl Instruction for Unregister<Domain> {}
    impl Instruction for Unregister<Account> {}
    impl Instruction for Unregister<AssetDefinition> {}
    impl Instruction for Unregister<Asset> {}
    impl Instruction for Unregister<Role> {}
    impl Instruction for Unregister<Trigger<TriggeringFilterBox>> {}

    impl Instruction for Mint<PublicKey, Account> {}
    impl Instruction for Mint<SignatureCheckCondition, Account> {}
    impl Instruction for Mint<u32, Asset> {}
    impl Instruction for Mint<u128, Asset> {}
    impl Instruction for Mint<Fixed, Asset> {}
    impl Instruction for Mint<u32, Trigger<TriggeringFilterBox>> {}

    impl Instruction for Burn<PublicKey, Account> {}
    impl Instruction for Burn<u32, Asset> {}
    impl Instruction for Burn<u128, Asset> {}
    impl Instruction for Burn<Fixed, Asset> {}
    impl Instruction for Burn<u32, Trigger<TriggeringFilterBox>> {}

    impl Instruction for Transfer<Account, DomainId, Account> {}
    impl Instruction for Transfer<Account, AssetDefinitionId, Account> {}
    impl Instruction for Transfer<Asset, u32, Account> {}
    impl Instruction for Transfer<Asset, u128, Account> {}
    impl Instruction for Transfer<Asset, Fixed, Account> {}

    impl Instruction for Grant<PermissionToken> {}
    impl Instruction for Grant<RoleId> {}

    impl Instruction for Revoke<PermissionToken> {}
    impl Instruction for Revoke<RoleId> {}

    impl Instruction for SetParameter {}
    impl Instruction for NewParameter {}
    impl Instruction for Upgrade {}
    impl Instruction for ExecuteTrigger {}
    impl Instruction for Log {}
    impl Instruction for Fail {}
}

mod transparent {
    use super::*;
    use crate::{account::NewAccount, domain::NewDomain};

    macro_rules! isi {
        ($($meta:meta)* $item:item) => {
            iroha_data_model_derive::model_single! {
                #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
                #[derive(getset::Getters)]
                #[derive(parity_scale_codec::Decode, parity_scale_codec::Encode)]
                #[derive(serde::Deserialize, serde::Serialize)]
                #[derive(iroha_schema::IntoSchema)]
                #[getset(get = "pub")]
                $($meta)*
                $item
            }
        };
    }

    macro_rules! impl_display {
        (
            $ty:ident $(< $($generic:tt),+ >)?
            $(where
                $( $lt:path $( : $clt:tt $(< $inner_generic:tt >)? $(+ $dlt:tt )* )? ),+ $(,)?)?
            => $fmt:literal, $($args:ident),* $(,)?
        ) => {
            impl $(< $($generic),+ >)? ::core::fmt::Display for $ty $(< $($generic),+ >)?
            $(where
                $( $lt $( : $clt $(< $inner_generic >)? $(+ $dlt )* )? ),+)?
            {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(
                        f,
                        $fmt,
                        $(self.$args),*
                    )
                }
            }
        }
    }

    macro_rules! impl_into_box {
        (
            $($isi:ident $(< $($generic:ident $(< $nested_generic:ident >)?),+ >)?)|*
            ==> $boxed:ident :: $variant:ident
        ) => {$(
            impl From<$isi $(< $($generic $(< $nested_generic >)?),+ >)? > for $boxed {
                fn from(instruction: $isi $(< $($generic $(< $nested_generic >)?),+ >)?) -> Self {
                    Self::$variant(instruction)
                }
            }
        )*};
        (
            $($isi:ident $(< $($generic:ident $(< $nested_generic:ident >)?),+ >)?)|*
            => $middle:ident ==> $boxed:ident :: $variant:ident
        ) => {$(
            impl From<$isi $(< $($generic $(< $nested_generic >)?),+ >)? > for $boxed {
                fn from(instruction: $isi $(< $($generic $(< $nested_generic >)?),+ >)?) -> Self {
                    Self::$variant($middle::from(instruction))
                }
            }
        )*};
    }

    isi! {
        /// Generic instruction for setting a chain-wide config parameter.
        #[derive(Constructor, Display)]
        #[display(fmt = "SET `{parameter}`")]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct SetParameter {
            /// The configuration parameter being changed.
            #[serde(flatten)]
            pub parameter: Parameter,
        }
    }

    impl_into_box!(SetParameter ==> InstructionBox::SetParameter);

    isi! {
        /// Sized structure for all possible on-chain configuration parameters when they are first created.
        /// Generic instruction for setting a chain-wide config parameter.
        #[derive(Constructor, Display)]
        #[display(fmt = "SET `{parameter}`")]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct NewParameter {
            /// Parameter to be changed.
            #[serde(flatten)]
            pub parameter: Parameter,
        }
    }

    impl_into_box!(NewParameter ==> InstructionBox::NewParameter);

    isi! {
        /// Generic instruction to set key value at the object.
        #[schema(bounds = "O: Identifiable, O::Id: IntoSchema")]
        pub struct SetKeyValue<O: Identifiable> {
            /// Where to set key value.
            #[serde(flatten)]
            pub object_id: O::Id,
            /// Key.
            pub key: Name,
            /// Value.
            pub value: Value,
        }
    }

    impl SetKeyValue<Domain> {
        /// Constructs a new [`SetKeyValue`] for a [`Domain`] with the given `key` and `value`.
        pub fn domain(domain_id: DomainId, key: Name, value: impl Into<Value>) -> Self {
            Self {
                object_id: domain_id,
                key,
                value: value.into(),
            }
        }
    }

    impl SetKeyValue<Account> {
        /// Constructs a new [`SetKeyValue`] for an [`Account`] with the given `key` and `value`.
        pub fn account(account_id: AccountId, key: Name, value: impl Into<Value>) -> Self {
            Self {
                object_id: account_id,
                key,
                value: value.into(),
            }
        }
    }

    impl SetKeyValue<AssetDefinition> {
        /// Constructs a new [`SetKeyValue`] for an [`AssetDefinition`] with the given `key` and `value`.
        pub fn asset_definition(
            asset_definition_id: AssetDefinitionId,
            key: Name,
            value: impl Into<Value>,
        ) -> Self {
            Self {
                object_id: asset_definition_id,
                key,
                value: value.into(),
            }
        }
    }

    impl SetKeyValue<Asset> {
        /// Constructs a new [`SetKeyValue`] for an [`Asset`] with the given `key` and `value`.
        pub fn asset(asset_id: AssetId, key: Name, value: impl Into<Value>) -> Self {
            Self {
                object_id: asset_id,
                key,
                value: value.into(),
            }
        }
    }

    impl_display! {
        SetKeyValue<O>
        where
            O: Identifiable,
            O::Id: Display,
        =>
        "SET `{}` = `{}` IN `{}`",
        key, value, object_id,
    }

    impl_into_box! {
        SetKeyValue<Domain> |
        SetKeyValue<Account> |
        SetKeyValue<AssetDefinition> |
        SetKeyValue<Asset> => SetKeyValueBox ==> InstructionBox::SetKeyValue
    }

    isi! {
        /// Generic instruction to remove key value at the object.
        #[schema(bounds = "O: Identifiable, O::Id: IntoSchema")]
        pub struct RemoveKeyValue<O: Identifiable> {
            /// From where to remove key value.
            #[serde(flatten)]
            pub object_id: O::Id,
            /// Key of the pair to remove.
            pub key: Name,
        }
    }

    impl RemoveKeyValue<Domain> {
        /// Constructs a new [`RemoveKeyValue`] for a [`Domain`] with the given `key`.
        pub fn domain(domain_id: DomainId, key: Name) -> Self {
            Self {
                object_id: domain_id,
                key,
            }
        }
    }

    impl RemoveKeyValue<Account> {
        /// Constructs a new [`RemoveKeyValue`] for an [`Account`] with the given `key`.
        pub fn account(account_id: AccountId, key: Name) -> Self {
            Self {
                object_id: account_id,
                key,
            }
        }
    }

    impl RemoveKeyValue<AssetDefinition> {
        /// Constructs a new [`RemoveKeyValue`] for an [`AssetDefinition`] with the given `key`.
        pub fn asset_definition(asset_definition_id: AssetDefinitionId, key: Name) -> Self {
            Self {
                object_id: asset_definition_id,
                key,
            }
        }
    }

    impl RemoveKeyValue<Asset> {
        /// Constructs a new [`RemoveKeyValue`] for an [`Asset`] with the given `key`.
        pub fn asset(asset_id: AssetId, key: Name) -> Self {
            Self {
                object_id: asset_id,
                key,
            }
        }
    }

    impl_display! {
        RemoveKeyValue<O>
        where
            O: Identifiable,
            O::Id: Display,
        =>
        "REMOVE `{}` from `{}`",
        key, object_id,
    }

    impl_into_box! {
        RemoveKeyValue<Domain> |
        RemoveKeyValue<Account> |
        RemoveKeyValue<AssetDefinition> |
        RemoveKeyValue<Asset> => RemoveKeyValueBox ==> InstructionBox::RemoveKeyValue
    }

    isi! {
        /// Generic instruction for a registration of an object to the identifiable destination.
        #[schema(bounds = "O: Registered, O::With: IntoSchema")]
        #[serde(transparent)]
        pub struct Register<O: Registered> {
            /// The object that should be registered, should be uniquely identifiable by its id.
            pub object: O::With,
        }
    }

    impl Register<Peer> {
        /// Constructs a new [`Register`] for a [`Peer`].
        pub fn peer(new_peer: Peer) -> Self {
            Self { object: new_peer }
        }
    }

    impl Register<Domain> {
        /// Constructs a new [`Register`] for a [`Domain`].
        pub fn domain(new_domain: NewDomain) -> Self {
            Self { object: new_domain }
        }
    }

    impl Register<Account> {
        /// Constructs a new [`Register`] for an [`Account`].
        pub fn account(new_account: NewAccount) -> Self {
            Self {
                object: new_account,
            }
        }
    }

    impl Register<AssetDefinition> {
        /// Constructs a new [`Register`] for an [`AssetDefinition`].
        pub fn asset_definition(new_asset_definition: NewAssetDefinition) -> Self {
            Self {
                object: new_asset_definition,
            }
        }
    }

    impl Register<Asset> {
        /// Constructs a new [`Register`] for an [`Asset`].
        pub fn asset(new_asset: Asset) -> Self {
            Self { object: new_asset }
        }
    }

    impl Register<Role> {
        /// Constructs a new [`Register`] for a [`Role`].
        pub fn role(new_role: NewRole) -> Self {
            Self { object: new_role }
        }
    }

    impl Register<Trigger<TriggeringFilterBox>> {
        /// Constructs a new [`Register`] for a [`Trigger`].
        pub fn trigger(new_trigger: Trigger<TriggeringFilterBox>) -> Self {
            Self {
                object: new_trigger,
            }
        }
    }

    impl_display! {
        Register<O>
        where
            O: Registered,
            O::With: Display,
        =>
        "REGISTER `{}`",
        object,
    }

    impl_into_box! {
        Register<Peer> |
        Register<Domain> |
        Register<Account> |
        Register<AssetDefinition> |
        Register<Asset> |
        Register<Role> |
        Register<Trigger<TriggeringFilterBox> > => RegisterBox ==> InstructionBox::Register
    }

    isi! {
        /// Generic instruction for an unregistration of an object from the identifiable destination.
        #[schema(bounds = "O: Identifiable, O::Id: IntoSchema")]
        pub struct Unregister<O: Identifiable> {
            /// [`Identifiable::Id`] of the object which should be unregistered.
            pub object_id: O::Id,
        }
    }

    impl_display! {
        Unregister<O>
        where
            O: Identifiable,
            O::Id: Display,
        =>
        "UNREGISTER `{}`",
        object_id,
    }

    impl_into_box! {
        Unregister<Peer> |
        Unregister<Domain> |
        Unregister<Account> |
        Unregister<AssetDefinition> |
        Unregister<Asset> |
        Unregister<Role> |
        Unregister<Trigger<TriggeringFilterBox> > => UnregisterBox ==> InstructionBox::Unregister
    }

    impl Unregister<Peer> {
        /// Constructs a new [`Unregister`] for a [`Peer`].
        pub fn peer(peer_id: PeerId) -> Self {
            Self { object_id: peer_id }
        }
    }

    impl Unregister<Domain> {
        /// Constructs a new [`Unregister`] for a [`Domain`].
        pub fn domain(domain_id: DomainId) -> Self {
            Self {
                object_id: domain_id,
            }
        }
    }

    impl Unregister<Account> {
        /// Constructs a new [`Unregister`] for an [`Account`].
        pub fn account(account_id: AccountId) -> Self {
            Self {
                object_id: account_id,
            }
        }
    }

    impl Unregister<AssetDefinition> {
        /// Constructs a new [`Unregister`] for an [`AssetDefinition`].
        pub fn asset_definition(asset_definition_id: AssetDefinitionId) -> Self {
            Self {
                object_id: asset_definition_id,
            }
        }
    }

    impl Unregister<Asset> {
        /// Constructs a new [`Unregister`] for an [`Asset`].
        pub fn asset(asset_id: AssetId) -> Self {
            Self {
                object_id: asset_id,
            }
        }
    }

    impl Unregister<Role> {
        /// Constructs a new [`Unregister`] for a [`Role`].
        pub fn role(role_id: RoleId) -> Self {
            Self { object_id: role_id }
        }
    }

    impl Unregister<Trigger<TriggeringFilterBox>> {
        /// Constructs a new [`Unregister`] for a [`Trigger`].
        pub fn trigger(trigger_id: TriggerId) -> Self {
            Self {
                object_id: trigger_id,
            }
        }
    }

    isi! {
        /// Generic instruction for a mint of an object to the identifiable destination.
        #[schema(bounds = "O: Into<Value> + IntoSchema, D: Identifiable, D::Id: IntoSchema")]
        pub struct Mint<O: Into<Value>, D: Identifiable> {
            /// Object which should be minted.
            pub object: O,
            /// Destination object [`Identifiable::Id`].
            pub destination_id: D::Id,
        }
    }

    impl Mint<PublicKey, Account> {
        /// Constructs a new [`Mint`] for a [`PublicKey`] for [`Account`].
        pub fn account_public_key(public_key: PublicKey, account_id: AccountId) -> Self {
            Self {
                object: public_key,
                destination_id: account_id,
            }
        }
    }

    impl Mint<SignatureCheckCondition, Account> {
        /// Constructs a new [`Mint`] for a [`SignatureCheckCondition`] for [`Account`].
        pub fn account_signature_check_condition(
            signature_check_condition: SignatureCheckCondition,
            account_id: AccountId,
        ) -> Self {
            Self {
                object: signature_check_condition,
                destination_id: account_id,
            }
        }
    }

    impl Mint<u32, Asset> {
        /// Constructs a new [`Mint`] for an [`Asset`] of [`Quantity`] type.
        pub fn asset_quantity(quantity: u32, asset_id: AssetId) -> Self {
            Self {
                object: quantity,
                destination_id: asset_id,
            }
        }
    }

    impl Mint<u128, Asset> {
        /// Constructs a new [`Mint`] for an [`Asset`] of [`BigQuantity`] type.
        pub fn asset_big_quantity(big_quantity: u128, asset_id: AssetId) -> Self {
            Self {
                object: big_quantity,
                destination_id: asset_id,
            }
        }
    }

    impl Mint<Fixed, Asset> {
        /// Constructs a new [`Mint`] for an [`Asset`] of [`Fixed`] type.
        pub fn asset_fixed(fixed: Fixed, asset_id: AssetId) -> Self {
            Self {
                object: fixed,
                destination_id: asset_id,
            }
        }
    }

    impl Mint<u32, Trigger<TriggeringFilterBox>> {
        /// Constructs a new [`Mint`] for repetition count of [`Trigger`].
        pub fn trigger_repetitions(repetitions: u32, trigger_id: TriggerId) -> Self {
            Self {
                object: repetitions,
                destination_id: trigger_id,
            }
        }
    }

    impl_display! {
        Mint<O, D>
        where
            O: Into<Value> + Display,
            D: Identifiable,
            D::Id: Display,
        =>
        "MINT `{}` TO `{}`",
        object,
        destination_id,
    }

    impl_into_box! {
        Mint<PublicKey, Account> |
        Mint<SignatureCheckCondition, Account> => AccountMintBox ==> MintBox::Account
    }

    impl_into_box! {
        Mint<u32, Asset> |
        Mint<u128, Asset> |
        Mint<Fixed, Asset> => AssetMintBox ==> MintBox::Asset
    }

    impl_into_box! {
        Mint<PublicKey, Account> |
        Mint<SignatureCheckCondition, Account> |
        Mint<u32, Asset> |
        Mint<u128, Asset> |
        Mint<Fixed, Asset> |
        Mint<u32, Trigger<TriggeringFilterBox> > => MintBox ==> InstructionBox::Mint
    }

    isi! {
        /// Generic instruction for a burn of an object to the identifiable destination.
        #[schema(bounds = "O: Into<Value> + IntoSchema, D: Identifiable, D::Id: IntoSchema")]
        pub struct Burn<O: Into<Value>, D: Identifiable> {
            /// Object which should be burned.
            pub object: O,
            /// Destination object [`Identifiable::Id`].
            pub destination_id: D::Id,
        }
    }

    impl Burn<PublicKey, Account> {
        /// Constructs a new [`Burn`] for a [`PublicKey`] for [`Account`].
        pub fn account_public_key(public_key: PublicKey, account_id: AccountId) -> Self {
            Self {
                object: public_key,
                destination_id: account_id,
            }
        }
    }

    impl Burn<u32, Asset> {
        /// Constructs a new [`Burn`] for an [`Asset`] of [`Quantity`] type.
        pub fn asset_quantity(quantity: u32, asset_id: AssetId) -> Self {
            Self {
                object: quantity,
                destination_id: asset_id,
            }
        }
    }

    impl Burn<u128, Asset> {
        /// Constructs a new [`Burn`] for an [`Asset`] of [`BigQuantity`] type.
        pub fn asset_big_quantity(big_quantity: u128, asset_id: AssetId) -> Self {
            Self {
                object: big_quantity,
                destination_id: asset_id,
            }
        }
    }

    impl Burn<Fixed, Asset> {
        /// Constructs a new [`Burn`] for an [`Asset`] of [`Fixed`] type.
        pub fn asset_fixed(fixed: Fixed, asset_id: AssetId) -> Self {
            Self {
                object: fixed,
                destination_id: asset_id,
            }
        }
    }

    impl Burn<u32, Trigger<TriggeringFilterBox>> {
        /// Constructs a new [`Burn`] for repetition count of [`Trigger`].
        pub fn trigger_repetitions(repetitions: u32, trigger_id: TriggerId) -> Self {
            Self {
                object: repetitions,
                destination_id: trigger_id,
            }
        }
    }

    impl_display! {
        Burn<O, D>
        where
            O: Into<Value> + Display,
            D: Identifiable,
            D::Id: Display,
        =>
        "BURN `{}` FROM `{}`",
        object,
        destination_id,
    }

    impl_into_box! {
        Burn<u32, Asset> |
        Burn<u128, Asset> |
        Burn<Fixed, Asset> => AssetBurnBox ==> BurnBox::Asset
    }

    impl_into_box! {
        Burn<PublicKey, Account> |
        Burn<u32, Asset> |
        Burn<u128, Asset> |
        Burn<Fixed, Asset> |
        Burn<u32, Trigger<TriggeringFilterBox> > => BurnBox ==> InstructionBox::Burn
    }

    isi! {
        /// Generic instruction for a transfer of an object from the identifiable source to the identifiable destination.
        #[schema(bounds = "S: Identifiable, S::Id: IntoSchema, \
                           O: Into<Value> + IntoSchema, \
                           D: Identifiable, D::Id: IntoSchema")]
        pub struct Transfer<S: Identifiable, O: Into<Value>, D: Identifiable> {
            /// Source object `Id`.
            pub source_id: S::Id,
            /// Object which should be transferred.
            pub object: O,
            /// Destination object `Id`.
            pub destination_id: D::Id,
        }
    }

    impl Transfer<Account, DomainId, Account> {
        /// Constructs a new [`Transfer`] for a [`Domain`].
        pub fn domain(from: AccountId, domain_id: DomainId, to: AccountId) -> Self {
            Self {
                source_id: from,
                object: domain_id,
                destination_id: to,
            }
        }
    }

    impl Transfer<Account, AssetDefinitionId, Account> {
        /// Constructs a new [`Transfer`] for an [`AssetDefinition`].
        pub fn asset_definition(
            from: AccountId,
            asset_definition_id: AssetDefinitionId,
            to: AccountId,
        ) -> Self {
            Self {
                source_id: from,
                object: asset_definition_id,
                destination_id: to,
            }
        }
    }

    impl Transfer<Asset, u32, Account> {
        /// Constructs a new [`Transfer`] for an [`Asset`] of [`Quantity`] type.
        pub fn asset_quantity(asset_id: AssetId, quantity: u32, to: AccountId) -> Self {
            Self {
                source_id: asset_id,
                object: quantity,
                destination_id: to,
            }
        }
    }

    impl Transfer<Asset, u128, Account> {
        /// Constructs a new [`Transfer`] for an [`Asset`] of [`BigQuantity`] type.
        pub fn asset_big_quantity(asset_id: AssetId, big_quantity: u128, to: AccountId) -> Self {
            Self {
                source_id: asset_id,
                object: big_quantity,
                destination_id: to,
            }
        }
    }

    impl Transfer<Asset, Fixed, Account> {
        /// Constructs a new [`Transfer`] for an [`Asset`] of [`Fixed`] type.
        pub fn asset_fixed(asset_id: AssetId, fixed: Fixed, to: AccountId) -> Self {
            Self {
                source_id: asset_id,
                object: fixed,
                destination_id: to,
            }
        }
    }

    impl_display! {
        Transfer<S, O, D>
        where
            S: Identifiable,
            S::Id: Display,
            O: Into<Value> + Display,
            D: Identifiable,
            D::Id: Display,
        =>
        "TRANSFER `{}` FROM `{}` TO `{}`",
        object,
        source_id,
        destination_id,
    }

    impl_into_box! {
        Transfer<Asset, u32, Account> |
        Transfer<Asset, u128, Account> |
        Transfer<Asset, Fixed, Account> => AssetTransferBox ==> TransferBox::Asset
    }

    impl_into_box! {
        Transfer<Account, DomainId, Account> |
        Transfer<Account, AssetDefinitionId, Account> |
        Transfer<Asset, u32, Account> |
        Transfer<Asset, u128, Account> |
        Transfer<Asset, Fixed, Account> => TransferBox ==> InstructionBox::Transfer
    }

    isi! {
        /// Utilitary instruction to fail execution and submit an error `message`.
        #[derive(Constructor, Display)]
        #[display(fmt = "FAIL `{message}`")]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct Fail {
            /// Message to submit.
            pub message: String,
        }
    }

    impl_into_box!(Fail ==> InstructionBox::Fail);

    isi! {
        /// Generic instruction for granting permission to an entity.
        pub struct Grant<O: Into<Value>> {
            /// Object to grant.
            pub object: O,
            /// Entity to which to grant this token.
            pub destination_id: AccountId,
        }
    }

    impl Grant<PermissionToken> {
        /// Constructs a new [`Grant`] for a [`PermissionToken`].
        pub fn permission(permission_token: PermissionToken, to: AccountId) -> Self {
            Self {
                object: permission_token,
                destination_id: to,
            }
        }
    }

    impl Grant<RoleId> {
        /// Constructs a new [`Grant`] for a [`Role`].
        pub fn role(role_id: RoleId, to: AccountId) -> Self {
            Self {
                object: role_id,
                destination_id: to,
            }
        }
    }

    impl_display! {
        Grant<O>
        where
            O: Into<Value> + Display,
        =>
        "GRANT `{}` TO `{}`",
        object,
        destination_id,
    }

    impl_into_box! {
        Grant<PermissionToken> |
        Grant<RoleId> => GrantBox ==> InstructionBox::Grant
    }

    isi! {
        /// Generic instruction for revoking permission from an entity.
        pub struct Revoke<O: Into<Value>> {
            /// Object to revoke.
            pub object: O,
            /// Entity which is being revoked this token from.
            pub destination_id: AccountId,
        }
    }

    impl Revoke<PermissionToken> {
        /// Constructs a new [`Revoke`] for a [`PermissionToken`].
        pub fn permission(permission_token: PermissionToken, from: AccountId) -> Self {
            Self {
                object: permission_token,
                destination_id: from,
            }
        }
    }

    impl Revoke<RoleId> {
        /// Constructs a new [`Revoke`] for a [`Role`].
        pub fn role(role_id: RoleId, from: AccountId) -> Self {
            Self {
                object: role_id,
                destination_id: from,
            }
        }
    }

    impl_display! {
        Revoke<O>
        where
            O: Into<Value> + Display,
        =>
        "REVOKE `{}` FROM `{}`",
        object,
        destination_id,
    }

    impl_into_box! {
        Revoke<PermissionToken> |
        Revoke<RoleId> => RevokeBox ==> InstructionBox::Revoke
    }

    isi! {
        /// Instruction to execute specified trigger
        #[derive(Constructor, Display)]
        #[display(fmt = "EXECUTE `{trigger_id}`")]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct ExecuteTrigger {
            /// Id of a trigger to execute
            pub trigger_id: TriggerId,
        }
    }

    impl_into_box!(ExecuteTrigger ==> InstructionBox::ExecuteTrigger);

    isi! {
        /// Generic instruction for upgrading runtime objects.
        #[derive(Constructor, Display)]
        #[display(fmt = "UPGRADE")]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct Upgrade {
            /// Object to upgrade.
            pub executor: Executor,
        }
    }

    impl_into_box!(Upgrade ==> InstructionBox::Upgrade);

    isi! {
        /// Instruction to print logs
        #[derive(Constructor, Display)]
        #[display(fmt = "LOG({level}): {msg}")]
        pub struct Log {
            /// Message log level
            #[serde(flatten)]
            pub level: Level,
            #[getset(skip)] // TODO: Fix this by addressing ffi issues
            /// Msg to be logged
            pub msg: String,
        }
    }

    impl_into_box!(Log ==> InstructionBox::Log);
}

macro_rules! isi_box {
    ($($meta:meta)* $item:item) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Display,
            parity_scale_codec::Decode,
            parity_scale_codec::Encode,
            serde::Deserialize,
            serde::Serialize,
            iroha_schema::IntoSchema,
            derive_more::From,
        )]
        $($meta)*
        $item
    };
}

isi_box! {
    /// Enum with all supported [`SetKeyValue`] instructions.
    pub enum SetKeyValueBox {
        /// Set key value for [`Domain`].
        Domain(SetKeyValue<Domain>),
        /// Set key value for [`Account`].
        Account(SetKeyValue<Account>),
        /// Set key value for [`AssetDefinition`].
        AssetDefinition(SetKeyValue<AssetDefinition>),
        /// Set key value for [`Asset`].
        Asset(SetKeyValue<Asset>),
    }
}

isi_box! {
    /// Enum with all supported [`RemoveKeyValue`] instructions.
    pub enum RemoveKeyValueBox {
        /// Remove key value from [`Domain`].
        Domain(RemoveKeyValue<Domain>),
        /// Remove key value from [`Account`].
        Account(RemoveKeyValue<Account>),
        /// Remove key value from [`AssetDefinition`].
        AssetDefinition(RemoveKeyValue<AssetDefinition>),
        /// Remove key value from [`Asset`].
        Asset(RemoveKeyValue<Asset>),
    }
}

isi_box! {
    /// Enum with all supported [`Register`] instructions.
    pub enum RegisterBox {
        /// Register [`Peer`].
        Peer(Register<Peer>),
        /// Register [`Domain`].
        Domain(Register<Domain>),
        /// Register [`Account`].
        Account(Register<Account>),
        /// Register [`AssetDefinition`].
        AssetDefinition(Register<AssetDefinition>),
        /// Register [`Asset`].
        Asset(Register<Asset>),
        /// Register [`Role`].
        Role(Register<Role>),
        /// Register [`Trigger`].
        Trigger(Register<Trigger<TriggeringFilterBox>>)
    }
}

isi_box! {
    /// Enum with all supported [`Unregister`] instructions.
    pub enum UnregisterBox {
        /// Unregister [`Peer`].
        Peer(Unregister<Peer>),
        /// Unregister [`Domain`].
        Domain(Unregister<Domain>),
        /// Unregister [`Account`].
        Account(Unregister<Account>),
        /// Unregister [`AssetDefinition`].
        AssetDefinition(Unregister<AssetDefinition>),
        /// Unregister [`Asset`].
        Asset(Unregister<Asset>),
        /// Unregister [`Role`].
        Role(Unregister<Role>),
        /// Unregister [`Trigger`].
        Trigger(Unregister<Trigger<TriggeringFilterBox>>)
    }
}

isi_box! {
    /// Enum with all supported [`Mint`] instructions.
    pub enum MintBox {
        /// Mint for [`Account`].
        Account(AccountMintBox),
        /// Mint for [`Asset`].
        Asset(AssetMintBox),
        /// Mint [`Trigger`] repetitions.
        TriggerRepetitions(Mint<u32, Trigger<TriggeringFilterBox>>),
    }
}

isi_box! {
    /// Enum with all supported [`Mint`] instructions related to [`Account`].
    pub enum AccountMintBox {
        /// Mint [`PublicKey`].
        PublicKey(Mint<PublicKey, Account>),
        /// Mint [`SignatureCheckCondition`].
        SignatureCheckCondition(Mint<SignatureCheckCondition, Account>),
    }
}

isi_box! {
    /// Enum with all supported [`Mint`] instructions related to [`Asset`].
    pub enum AssetMintBox {
        /// Mint [`Asset`] of [`Quantity`] type.
        Quantity(Mint<u32, Asset>),
        /// Mint [`Asset`] of [`BigQuantity`] type.
        BigQuantity(Mint<u128, Asset>),
        /// Mint [`Asset`] of [`Fixed`] type.
        Fixed(Mint<Fixed, Asset>),
    }
}

isi_box! {
    /// Enum with all supported [`Burn`] instructions.
    pub enum BurnBox {
        /// Burn [`PublicKey`] for [`Account`].
        AccountPublicKey(Burn<PublicKey, Account>),
        /// Burn [`Asset`].
        Asset(AssetBurnBox),
        /// Burn [`Trigger`] repetitions.
        TriggerRepetitions(Burn<u32, Trigger<TriggeringFilterBox>>),
    }
}

isi_box! {
    /// Enum with all supported [`Burn`] instructions related to [`Asset`].
    pub enum AssetBurnBox {
        /// Burn [`Asset`] of [`Quantity`] type.
        Quantity(Burn<u32, Asset>),
        /// Burn [`Asset`] of [`BigQuantity`] type.
        BigQuantity(Burn<u128, Asset>),
        /// Burn [`Asset`] of [`Fixed`] type.
        Fixed(Burn<Fixed, Asset>),
    }
}

isi_box! {
    /// Enum with all supported [`Transfer`] instructions.
    pub enum TransferBox {
        /// Transfer [`Domain`] to another [`Account`].
        Domain(Transfer<Account, DomainId, Account>),
        /// Transfer [`AssetDefinition`] to another [`Account`].
        AssetDefinition(Transfer<Account, AssetDefinitionId, Account>),
        /// Transfer [`Asset`] to another [`Account`].
        Asset(AssetTransferBox),
    }
}

isi_box! {
    /// Enum with all supported [`Transfer`] instructions related to [`Asset`].
    pub enum AssetTransferBox {
        /// Transfer [`Asset`] of [`Quantity`] type.
        Quantity(Transfer<Asset, u32, Account>),
        /// Transfer [`Asset`] of [`BigQuantity`] type.
        BigQuantity(Transfer<Asset, u128, Account>),
        /// Transfer [`Asset`] of [`Fixed`] type.
        Fixed(Transfer<Asset, Fixed, Account>),
    }
}

isi_box! {
    /// Enum with all supported [`Grant`] instructions.
    pub enum GrantBox {
        /// Grant [`PermissionToken`] to [`Account`].
        PermissionToken(Grant<PermissionToken>),
        /// Grant [`Role`] to [`Account`].
        Role(Grant<RoleId>),
    }
}

isi_box! {
    /// Enum with all supported [`Revoke`] instructions.
    pub enum RevokeBox {
        /// Revoke [`PermissionToken`] from [`Account`].
        PermissionToken(Revoke<PermissionToken>),
        /// Revoke [`Role`] from [`Account`].
        Role(Revoke<RoleId>),
    }
}

pub mod error {
    //! Module containing errors that can occur during instruction evaluation

    #[cfg(not(feature = "std"))]
    use alloc::{boxed::Box, format, string::String, vec::Vec};
    use core::fmt::Debug;

    use derive_more::Display;
    use iroha_data_model_derive::model;
    use iroha_macro::FromVariant;
    use iroha_primitives::fixed::FixedPointOperationError;
    use iroha_schema::IntoSchema;
    use parity_scale_codec::{Decode, Encode};

    pub use self::model::*;
    use super::InstructionType;
    use crate::{
        asset::AssetValueType,
        metadata,
        query::error::{FindError, QueryExecutionFail},
        IdBox, Value,
    };

    #[model]
    pub mod model {
        use serde::{Deserialize, Serialize};

        use super::*;
        use crate::asset::AssetDefinitionId;

        /// Instruction execution error type
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            FromVariant,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[ignore_extra_doc_attributes]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        // TODO: Only temporarily opaque because of InstructionExecutionError::Repetition
        #[ffi_type(opaque)]
        pub enum InstructionExecutionError {
            /// Instruction does not adhere to Iroha DSL specification
            Evaluate(#[cfg_attr(feature = "std", source)] InstructionEvaluationError),
            /// Query failed
            Query(#[cfg_attr(feature = "std", source)] QueryExecutionFail),
            /// Conversion Error: {0}
            Conversion(
                #[skip_from]
                #[skip_try_from]
                String,
            ),
            /// Entity missing
            Find(#[cfg_attr(feature = "std", source)] FindError),
            /// Repeated instruction
            Repetition(#[cfg_attr(feature = "std", source)] RepetitionError),
            /// Mintability assertion failed
            Mintability(#[cfg_attr(feature = "std", source)] MintabilityError),
            /// Illegal math operation
            Math(#[cfg_attr(feature = "std", source)] MathError),
            /// Metadata error
            Metadata(#[cfg_attr(feature = "std", source)] metadata::MetadataError),
            /// Execution failed: {0}
            Fail(
                #[skip_from]
                #[skip_try_from]
                String,
            ),
            /// Invalid instruction parameter
            InvalidParameter(#[cfg_attr(feature = "std", source)] InvalidParameterError),
            /// Iroha invariant violation: {0}
            ///
            /// i.e. you can't burn last key
            InvariantViolation(
                #[skip_from]
                #[skip_try_from]
                String,
            ),
        }

        /// Evaluation error. This error indicates instruction is not a valid Iroha DSL
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            FromVariant,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        // TODO: Only temporarily opaque because of problems with FFI
        #[ffi_type(opaque)]
        pub enum InstructionEvaluationError {
            /// Unsupported parameter type for instruction of type `{0}`
            Unsupported(InstructionType),
            /// Failed to find parameter in a permission: {0}
            PermissionParameter(String),
            /// Incorrect value type
            Type(#[cfg_attr(feature = "std", source)] TypeError),
        }

        /// Generic structure used to represent a mismatch
        #[derive(
            Debug,
            Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[display(fmt = "Expected {expected:?}, actual {actual:?}")]
        #[ffi_type]
        pub struct Mismatch<T: Debug> {
            /// The value that is needed for normal execution
            pub expected: T,
            /// The value that caused the error
            pub actual: T,
        }

        /// Type error
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            FromVariant,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[ffi_type]
        pub enum TypeError {
            /// Asset Ids correspond to assets with different underlying types, {0}
            AssetValueType(#[cfg_attr(feature = "std", source)] Mismatch<AssetValueType>),
            /// Value passed to the parameter doesn't have the right type, {0}
            ParameterValueType(#[cfg_attr(feature = "std", source)] Box<Mismatch<Value>>),
            /// AssetDefinition Ids don't match, {0}
            AssetDefinitionId(
                #[cfg_attr(feature = "std", source)] Box<Mismatch<AssetDefinitionId>>,
            ),
        }

        /// Math error, which occurs during instruction execution
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            FromVariant,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        // TODO: Only temporarily opaque because of InstructionExecutionError::BinaryOpIncompatibleNumericValueTypes
        #[ignore_extra_doc_attributes]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[ffi_type(opaque)]
        pub enum MathError {
            /// Overflow error occurred inside instruction
            Overflow,
            /// Not enough quantity to transfer/burn
            NotEnoughQuantity,
            /// Divide by zero
            DivideByZero,
            /// Negative value encountered
            NegativeValue,
            /// Domain violation
            DomainViolation,
            /// Unknown error
            ///
            /// No actual function should ever return this if possible
            Unknown,
            /// Conversion failed: {0}
            FixedPointConversion(String),
        }

        /// Mintability logic error
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[ffi_type]
        #[repr(u8)]
        pub enum MintabilityError {
            /// This asset cannot be minted more than once and it was already minted
            MintUnmintable,
            /// This asset was set as infinitely mintable. You cannot forbid its minting
            ForbidMintOnMintable,
        }

        /// Invalid instruction parameter error
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[ignore_extra_doc_attributes]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[ffi_type(opaque)]
        #[repr(u8)]
        pub enum InvalidParameterError {
            /// Invalid WASM binary: {0}
            Wasm(String),
            /// Name length violation
            ///
            /// i.e. too long [`AccountId`]
            NameLength,
        }

        /// Repetition of of `{instruction_type}` for id `{id}`
        #[derive(
            Debug,
            displaydoc::Display,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Deserialize,
            Serialize,
            Decode,
            Encode,
            IntoSchema,
        )]
        #[cfg_attr(feature = "std", derive(thiserror::Error))]
        #[ffi_type]
        pub struct RepetitionError {
            /// Instruction type
            pub instruction_type: InstructionType,
            /// Id of the object being repeated
            pub id: IdBox,
        }
    }

    impl From<TypeError> for InstructionExecutionError {
        fn from(err: TypeError) -> Self {
            Self::Evaluate(InstructionEvaluationError::Type(err))
        }
    }
    impl From<FixedPointOperationError> for MathError {
        fn from(err: FixedPointOperationError) -> Self {
            match err {
                FixedPointOperationError::NegativeValue(_) => Self::NegativeValue,
                FixedPointOperationError::Conversion(e) => {
                    #[cfg(not(feature = "std"))]
                    use alloc::string::ToString as _;

                    Self::FixedPointConversion(e.to_string())
                }
                FixedPointOperationError::Overflow => Self::Overflow,
                FixedPointOperationError::DivideByZero => Self::DivideByZero,
                FixedPointOperationError::DomainViolation => Self::DomainViolation,
                FixedPointOperationError::Arithmetic => Self::Unknown,
            }
        }
    }
}

/// The prelude re-exports most commonly used traits, structs and macros from this crate.
pub mod prelude {
    pub use super::{
        AccountMintBox, AssetBurnBox, AssetMintBox, AssetTransferBox, Burn, BurnBox,
        ExecuteTrigger, Fail, Grant, GrantBox, InstructionBox, Log, Mint, MintBox, NewParameter,
        Register, RegisterBox, RemoveKeyValue, RemoveKeyValueBox, Revoke, RevokeBox, SetKeyValue,
        SetKeyValueBox, SetParameter, Transfer, TransferBox, Unregister, UnregisterBox, Upgrade,
    };
}
