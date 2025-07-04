use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::error::{Error, Result};
use crate::model::ModelManager;
use crate::pwd;
use crate::pwd::ContentToHash;
use hmac::digest::typenum::Mod;
use modql::field::{Fields, HasSeaFields};
use sea_query::{Expr, PostgresQueryBuilder, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;


/// Generic user type
///
/// Since we want to serialize the user, it is important to not have the password.
#[derive(Clone, Debug, Serialize, FromRow, Fields)]
pub struct User {
    pub id: i64,
    pub username: String,
}


/// This is what is being sent from the client or from the server API
#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}


/// For when we insert a new user, more of an implementation detail.
#[derive(Fields)]
struct UserForInsert {
    username: String,
}


/// For the login logic.
///
/// Read-only: have the information necessary to validate the login
#[derive(Clone, Debug, FromRow, Fields)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,
    pub pwd: Option<String>, // encrypted, #_scheme_id_#...
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}


/// For the authentication logic.
#[derive(Clone, Debug, FromRow, Fields)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,
    pub token_salt: Uuid,
}


/// Marker trait: in a way doesn't add any information, but groups the three struct together, so UserBmc::get
/// can only return those.
pub trait UserBy: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}


impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}


/// Iden allows us to use those as column names.
///
/// These do not need to be exhaustive with respect to the modql::fields::Fields, we only need
/// the ones we use manually in our queries.
#[derive(sea_query::Iden)]
enum UserIden {
    Id,
    Username,
    Pwd,
}


pub struct UserBmc {}

impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        // Build query
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(E::sea_idens())
            .and_where(Expr::col(UserIden::Username).eq(username));

        // Execute query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let user = sqlx::query_as_with::<_, E, _>(&sql, values)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }

    pub async fn update_pwd(ctx: &Ctx, mm: &ModelManager, id: i64, pwd_clear: &str) -> Result<()> {
        let db = mm.db();

        // Prepare password
        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let pwd = pwd::hash_pwd(&ContentToHash {
            content: pwd_clear.to_string(),
            salt: user.pwd_salt,
        })?;

        // Build query
        let mut query = Query::update();
        query
            .table(Self::table_ref())
            .value(UserIden::Pwd, SimpleExpr::from(pwd))
            .and_where(Expr::col(UserIden::Id).eq(user.id));

        // Execute query
        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let _count = sqlx::query_with(&sql, values)
            .execute(db)
            .await?
            .rows_affected();

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        // Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        // Execute
        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context("Should have user 'demo1'")?;

        // Check
        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
