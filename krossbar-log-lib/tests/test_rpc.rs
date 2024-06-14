use log::*;
use rstest::rstest;

mod fixture;
use fixture::{make_fixture, Fixture};

#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
async fn test_simple_log(
    #[from(make_fixture)]
    #[future]
    fixture: Fixture,
) {
    // error!("Error message");
    // warn!("Warning message");
    // info!("Info message");
    // debug!("Debug message");
    // trace!("Trace message");

    fixture.cancel();
}
