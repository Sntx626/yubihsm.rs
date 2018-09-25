//! Integration tests (using live YubiHSM2 or MockHsm)

#[cfg(not(feature = "mockhsm"))]
#[macro_use]
extern crate lazy_static;
extern crate ring;
extern crate sha2;
extern crate untrusted;
extern crate yubihsm;
use yubihsm::{AsymmetricAlg, Capability, Client, Domain, ObjectId, ObjectType};

/// Perform a live integration test against yubihsm-connector and a real `YubiHSM2`
#[cfg(not(feature = "mockhsm"))]
macro_rules! create_session {
    () => {
        $crate::HSM_CLIENT.lock().unwrap()
    };
}

/// Perform an integration test against the MockHsm (useful for CI)
#[cfg(feature = "mockhsm")]
macro_rules! create_session {
    () => {
        $crate::TestClient::create(::yubihsm::mockhsm::MockHsm::new(), Default::default(), true)
            .unwrap()
    };
}

#[cfg(not(any(feature = "usb", feature = "mockhsm")))]
lazy_static! {
    static ref HSM_CLIENT: ::std::sync::Mutex<TestClient> = {
        let session = Client::create(Default::default(), Default::default(), true)
            .unwrap_or_else(|err| panic!("{}", err));
        ::std::sync::Mutex::new(session)
    };
}

#[cfg(all(feature = "usb", not(feature = "mockhsm")))]
lazy_static! {
    static ref HSM_CLIENT: ::std::sync::Mutex<TestClient> = {
        let session = Client::create(Default::default(), Default::default(), true)
            .unwrap_or_else(|err| panic!("{}", err));
        ::std::sync::Mutex::new(session)
    };
}

/// Integration tests for individual YubiHSM2 commands
pub mod command;

/// Cryptographic test vectors taken from standards documents
mod test_vectors;

#[cfg(not(any(feature = "usb", feature = "mockhsm")))]
use yubihsm::HttpAdapter;

#[cfg(all(feature = "usb", not(feature = "mockhsm")))]
use yubihsm::UsbAdapter;

#[cfg(feature = "mockhsm")]
use yubihsm::mockhsm::MockAdapter;

#[cfg(not(any(feature = "usb", feature = "mockhsm")))]
type TestClient = Client<HttpAdapter>;

#[cfg(all(feature = "usb", not(feature = "mockhsm")))]
type TestClient = Client<UsbAdapter>;

#[cfg(feature = "mockhsm")]
type TestClient = Client<MockAdapter>;

/// Key ID to use for testing keygen/signing
const TEST_KEY_ID: ObjectId = 100;

/// Key ID to use as a keywrapping subject
const TEST_EXPORTED_KEY_ID: ObjectId = 101;

/// Label to use for the test key
const TEST_KEY_LABEL: &str = "yubihsm.rs test key";

/// Label for the default auth key
const DEFAULT_AUTH_KEY_LABEL: &str = "DEFAULT AUTHKEY CHANGE THIS ASAP";

/// Label to use for the exported test
const TEST_EXPORTED_KEY_LABEL: &str = "yubihsm.rs exported test key";

/// Domain to use for all tests
const TEST_DOMAINS: Domain = Domain::DOM1;

/// Message to sign when performing tests
const TEST_MESSAGE: &[u8] = b"The YubiHSM2 is a simple, affordable, and secure HSM solution";

/// Size of a NIST P-256 public key
pub const EC_P256_PUBLIC_KEY_SIZE: usize = 64;

/// Ensure we can read the YubiHSM2's serial number
#[test]
fn get_yubihsm_serial_number() {
    let mut session = create_session!();
    #[allow(unused_variables)]
    let serial_result = session.serial_number();
    #[cfg(any(feature = "usb", feature = "mockhsm"))]
    assert!(serial_result.is_ok());
}

//
// Helper Functions
//

/// Delete the key in the test key slot (if it exists, otherwise do nothing)
pub fn clear_test_key_slot(session: &mut TestClient, object_type: ObjectType) {
    // Delete the key in TEST_KEY_ID slot it exists (we use it for testing)
    // Ignore errors since the object may not exist yet
    let _ = yubihsm::delete_object(session, TEST_KEY_ID, object_type);

    // Ensure the object does not already exist
    assert!(yubihsm::get_object_info(session, TEST_KEY_ID, object_type).is_err());
}

/// Create a public key for use in a test
pub fn generate_asymmetric_key(
    session: &mut TestClient,
    algorithm: AsymmetricAlg,
    capabilities: Capability,
) {
    clear_test_key_slot(session, ObjectType::AsymmetricKey);

    let key_id = yubihsm::generate_asymmetric_key(
        session,
        TEST_KEY_ID,
        TEST_KEY_LABEL.into(),
        TEST_DOMAINS,
        capabilities,
        algorithm,
    ).unwrap_or_else(|err| panic!("error generating asymmetric key: {}", err));

    assert_eq!(key_id, TEST_KEY_ID);
}

/// Put an asymmetric private key into the HSM
pub fn put_asymmetric_key<T: Into<Vec<u8>>>(
    session: &mut TestClient,
    algorithm: AsymmetricAlg,
    capabilities: Capability,
    data: T,
) {
    clear_test_key_slot(session, ObjectType::AsymmetricKey);

    let key_id = yubihsm::put_asymmetric_key(
        session,
        TEST_KEY_ID,
        TEST_KEY_LABEL.into(),
        TEST_DOMAINS,
        capabilities,
        algorithm,
        data,
    ).unwrap_or_else(|err| panic!("error putting asymmetric key: {}", err));

    assert_eq!(key_id, TEST_KEY_ID);
}
