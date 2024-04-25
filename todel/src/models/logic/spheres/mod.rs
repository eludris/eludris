use crate::models::Sphere;

impl Sphere {
    pub async fn create(
        sphere: SphereCreate,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
    }
}
