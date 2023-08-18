/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2023 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_std::{net::TcpListener, sync::Arc, task};
use async_trait::async_trait;
use serde_json::{json, Value};
use smol::channel::{Receiver, Sender};
use url::Url;

use darkfi::{
    net::transport::Listener,
    rpc::{
        client::RpcClient,
        jsonrpc::*,
        server::{accept, RequestHandler},
    },
    Result,
};

struct RpcSrv {
    stop_sub: (Sender<()>, Receiver<()>),
}

impl RpcSrv {
    async fn pong(&self, id: Value, _params: &[Value]) -> JsonResult {
        JsonResponse::new(json!("pong"), id).into()
    }

    async fn kill(&self, id: Value, _params: &[Value]) -> JsonResult {
        self.stop_sub.0.send(()).await.unwrap();
        JsonResponse::new(json!("bye"), id).into()
    }
}

#[async_trait]
impl RequestHandler for RpcSrv {
    async fn handle_request(&self, req: JsonRequest) -> JsonResult {
        let params = req.params.as_array().unwrap();

        match req.method.as_str() {
            Some("ping") => return self.pong(req.id, params).await,
            Some("kill") => return self.kill(req.id, params).await,
            Some(_) | None => return JsonError::new(ErrorCode::MethodNotFound, None, req.id).into(),
        }
    }
}

#[async_std::test]
async fn jsonrpc_reqrep() -> Result<()> {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let sockaddr = listener.local_addr()?;
    let endpoint = Url::parse(&format!("tcp://127.0.0.1:{}", sockaddr.port()))?;
    drop(listener);

    let rpcsrv = Arc::new(RpcSrv { stop_sub: smol::channel::unbounded() });
    let listener = Listener::new(endpoint.clone()).await?.listen().await?;

    task::spawn(async move {
        while let Ok((stream, peer_addr)) = listener.next().await {
            let _rh = rpcsrv.clone();
            task::spawn(async move {
                let _ = accept(stream, peer_addr.clone(), _rh).await;
            });
        }
    });

    let client = RpcClient::new(endpoint, None).await?;
    let req = JsonRequest::new("ping", json!([]));
    let rep = client.request(req).await?;

    let rep = rep.as_str().unwrap();
    assert_eq!(rep, "pong");

    let req = JsonRequest::new("kill", json!([]));
    let rep = client.request(req).await?;

    let rep = rep.as_str().unwrap();
    assert_eq!(rep, "bye");

    Ok(())
}
