use std::{borrow::Cow, fmt::Debug, path::Path, sync::atomic::AtomicU64};

use alloy_json_rpc::{Request, Response, ResponsePayload, RpcSend, SerializedRequest};

use crate::re_ipc_transport::ReIPC;

pub struct RpcProvider {
    id: AtomicU64,
    ipc: ReIPC,
}

impl RpcProvider {
    pub fn try_connect(path: &Path) -> anyhow::Result<Self> {
        let ipc = ReIPC::try_connect(path)?;

        Ok(Self {
            ipc,
            id: Default::default(),
        })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.ipc.close()
    }

    pub fn call<ReqParams, Resp>(
        &self,
        method: impl Into<Cow<'static, str>>,
        params: ReqParams,
    ) -> anyhow::Result<Resp>
    where
        ReqParams: RpcSend,
        Resp: Debug + serde::de::DeserializeOwned,
    {
        let req = self.make_request(method, params);
        let resp = self.ipc.call(req)?;
        RpcProvider::parse_response(resp)
    }

    pub fn call_no_params<Resp>(&self, method: impl Into<Cow<'static, str>>) -> anyhow::Result<Resp>
    where
        Resp: Debug + serde::de::DeserializeOwned,
    {
        let req = self.make_request(method, ());
        let resp = self.ipc.call(req)?;
        RpcProvider::parse_response(resp)
    }

    fn make_request<P: RpcSend>(
        &self,
        method: impl Into<Cow<'static, str>>,
        params: P,
    ) -> SerializedRequest {
        // Relaxed is ok, because we are not syncronizing anything,
        // atomicitiy&fetch_add guarantees that numbers will be in stricly increasing order
        // thusly unique
        let id = self.id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let req = Request::new(method, id.into(), params);
        req.try_into().unwrap()
    }

    fn parse_response<T>(resp: Response) -> anyhow::Result<T>
    where
        T: Debug + serde::de::DeserializeOwned,
    {
        if let ResponsePayload::Success(_) = resp.payload {
            if let Some(Ok(data)) = resp.try_success_as::<T>() {
                return Ok(data);
            } else {
                //TODO: add tracing crate
                //println!("Failed to deserialize success payload");
            }
        };

        anyhow::bail!("Failed to get response")
    }
}
