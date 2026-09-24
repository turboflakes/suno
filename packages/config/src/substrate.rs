use codec::Encode;
use scale_info::PortableRegistry;
use subxt::{
    config::{
        substrate::{DynamicHasher256, SubstrateHeader},
        transaction_extensions::{
            self, ChargeAssetTxPaymentParams, ChargeTransactionPaymentParams, CheckMortalityParams,
            CheckNonceParams,
        },
        ClientState, Hasher, TransactionExtension, TransactionExtensions,
    },
    error::TransactionExtensionError,
    ext::frame_decode::extrinsics::{
        TransactionExtension as TransactionExtensionExt,
        TransactionExtensionError as TransactionExtensionErrorExt,
    },
    utils::{AccountId32, MultiAddress, MultiSignature},
};

// Copy of the default [`subxt::config::CustomConfig`] customized with new transaction extensions.
#[derive(Debug, Clone, Default)]
pub struct CustomConfig {}

impl subxt::Config for CustomConfig {
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Signature = MultiSignature;
    type Hasher = DynamicHasher256;
    type Header = SubstrateHeader<<Self::Hasher as Hasher>::Hash>;
    type AssetId = u32;

    // Override only TransactionExtensions to add the new extensions
    type TransactionExtensions = SubstrateExtrinsicParams<Self>;
}

pub type SubstrateExtrinsicParams<T> = (
    transaction_extensions::VerifySignature<T>,
    transaction_extensions::CheckSpecVersion,
    transaction_extensions::CheckTxVersion,
    transaction_extensions::CheckNonce,
    transaction_extensions::CheckGenesis<T>,
    transaction_extensions::CheckMortality<T>,
    transaction_extensions::ChargeAssetTxPayment<T>,
    transaction_extensions::ChargeTransactionPayment,
    transaction_extensions::CheckMetadataHash,
    RestrictOrigins,
);

// Mirrors subxt's [`DefaultExtrinsicParamsBuilder`], building the parameters for
// [`SubstrateExtrinsicParams`] directly.
pub struct CustomExtrinsicParamsBuilder<T: subxt::Config> {
    nonce: CheckNonceParams,
    mortality: CheckMortalityParams<T>,
    tip: u128,
}

impl<T: subxt::Config> Default for CustomExtrinsicParamsBuilder<T> {
    fn default() -> Self {
        Self {
            nonce: CheckNonceParams::from_chain(),
            mortality: CheckMortalityParams::default(),
            tip: 0,
        }
    }
}

impl<T: subxt::Config> CustomExtrinsicParamsBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = CheckNonceParams::with_nonce(nonce);
        self
    }

    pub fn mortal(mut self, for_n_blocks: u64) -> Self {
        self.mortality = CheckMortalityParams::mortal(for_n_blocks);
        self
    }

    pub fn tip(mut self, tip: u128) -> Self {
        self.tip = tip;
        self
    }

    pub fn build(self) -> <SubstrateExtrinsicParams<T> as TransactionExtensions<T>>::Params {
        (
            (),
            (),
            (),
            self.nonce,
            (),
            self.mortality,
            ChargeAssetTxPaymentParams::tip(self.tip),
            ChargeTransactionPaymentParams::tip(self.tip),
            (),
            // Additional extensions take no parameters.
            (),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct RestrictOrigins(bool);

impl<T: subxt::Config> TransactionExtension<T> for RestrictOrigins {
    type Decoded = bool;
    type Params = ();

    fn new(
        _client: &ClientState<T>,
        _params: Self::Params,
    ) -> Result<Self, TransactionExtensionError> {
        Ok(Self(true))
    }
}

impl TransactionExtensionExt<PortableRegistry> for RestrictOrigins {
    const NAME: &str = "RestrictOrigins";

    fn encode_value_to(
        &self,
        _type_id: u32,
        _type_resolver: &PortableRegistry,
        out: &mut Vec<u8>,
    ) -> Result<(), TransactionExtensionErrorExt> {
        self.0.encode_to(out);
        Ok(())
    }

    fn encode_implicit_to(
        &self,
        _type_id: u32,
        _type_resolver: &PortableRegistry,
        _out: &mut Vec<u8>,
    ) -> Result<(), TransactionExtensionErrorExt> {
        Ok(())
    }
}
