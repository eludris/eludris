use chrono::{Duration, Utc};
use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use rocket_db_pools::Database;
use todel::{http::DB, models::User};
use tokio::time::sleep;

pub struct ScheduledCleanup;

#[rocket::async_trait]
impl Fairing for ScheduledCleanup {
    fn info(&self) -> Info {
        Info {
            name: "Handle creating a scheduled cleaning up task",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let mut db = {
            let pool = DB::fetch(&rocket).expect("Could not get the managed pool");
            pool.acquire()
                .await
                .expect("Failed to acquire database connection")
        };
        tokio::spawn(async move {
            let now = Utc::now().naive_utc();
            let midnight = (now + Duration::days(1))
                .date()
                .and_hms_opt(0, 0, 0)
                .expect("Couldn't get next midnight");
            let first_sleep = midnight
                .signed_duration_since(now)
                .to_std()
                .expect("Couldn't determine how many seconds there are until next midnight");
            sleep(first_sleep).await;
            loop {
                log::info!("Running scheduled cleanup");
                if let Err(err) = User::clean_up_unverified(&mut db).await {
                    log::error!("Couldn't clean up unverified users: {}", err);
                }
                if let Err(err) = User::clean_up_deleted(&mut db).await {
                    log::error!("Couldn't clean up deleted users: {}", err);
                }
                sleep(
                    Duration::days(1)
                        .to_std()
                        .expect("Couldn't convery chrono Duration to std Duration"),
                )
                .await;
            }
        });
        Ok(rocket)
    }
}
