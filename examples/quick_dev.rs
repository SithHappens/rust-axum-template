use anyhow::Result;
use serde_json::json;


#[tokio::main]
async fn main() -> Result<()> {
    let client = httpc_test::new_client("http://localhost:8080")?;
    //client.do_get("/index.html").await?.print().await?;

    // Login
    let req_login = client.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "welcome"
        }),
    );
    req_login.await?.print().await?;

    // RPC
    let req_create_task = client.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "create_task",
            "params": {
                "data": {
                    "title": "task AAA",
                }
            }
        }),
    );
    req_create_task.await?.print().await?;

    let req_update_task = client.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "update_task",
            "params": {
                "id": 1000,
                "data": {
                    "title": "task BB",
                }
            }
        }),
    );
    req_update_task.await?.print().await?;

    let req_list_tasks = client.do_post("/api/rpc", json!({"id": 1, "method": "list_tasks"}));
    req_list_tasks.await?.print().await?;

    let req_delete_task = client.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "delete_task",
            "params": {
                "id": 1001,
            }
        }),
    );
    req_delete_task.await?.print().await?;

    // Logoff
    let req_logoff = client.do_post(
        "/api/logoff",
        json!({
            "logoff": true
        }),
    );
    //req_logoff.await?.print().await?;

    Ok(())
}
