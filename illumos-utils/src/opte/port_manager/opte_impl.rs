// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::opte::Error;
use oxide_vpc::api::AddRouterEntryReq;
use oxide_vpc::api::DelRouterEntryReq;
use oxide_vpc::api::DelRouterEntryResp;
use oxide_vpc::api::DhcpCfg;
use oxide_vpc::api::Direction;
use oxide_vpc::api::IpCidr;
use oxide_vpc::api::SetFwRulesReq;
use oxide_vpc::api::VpcCfg;

pub(super) trait OpteImpl: Sized {
    fn open() -> Result<Self, Error>;

    fn create_xde(
        &self,
        name: &str,
        vpc: VpcCfg,
        dhcp: DhcpCfg,
        passthrough: bool,
    ) -> Result<(), Error>;

    fn set_fw_rules(&self, req: &SetFwRulesReq) -> Result<(), Error>;

    fn add_router_entry(&self, req: &AddRouterEntryReq) -> Result<(), Error>;

    fn del_router_entry(
        &self,
        req: &DelRouterEntryReq,
    ) -> Result<DelRouterEntryResp, Error>;

    fn allow_cidr(
        &self,
        port_name: &str,
        cidr: IpCidr,
        dir: Direction,
    ) -> Result<(), Error>;
}

#[cfg(target_os = "illumos")]
pub(super) use self::illumos::OpteImplIllumos as OpteImplPlatform;
#[cfg(not(target_os = "illumos"))]
pub(super) use self::noop::OpteImplNoop as OpteImplPlatform;

#[cfg(target_os = "illumos")]
mod illumos {
    use super::*;
    use opte_ioctl::OpteHdl;

    pub(crate) struct OpteImplIllumos {
        hdl: OpteHdl,
    }

    impl OpteImpl for OpteImplIllumos {
        fn open() -> Result<Self, Error> {
            let hdl = OpteHdl::open(OpteHdl::XDE_CTL)?;
            Ok(Self { hdl })
        }

        fn create_xde(
            &self,
            name: &str,
            vpc: VpcCfg,
            dhcp: DhcpCfg,
            passthrough: bool,
        ) -> Result<(), Error> {
            self.hdl.create_xde(name, vpc, dhcp, passthrough)?;
            Ok(())
        }

        fn set_fw_rules(&self, req: &SetFwRulesReq) -> Result<(), Error> {
            self.hdl.set_fw_rules(req)?;
            Ok(())
        }

        fn add_router_entry(
            &self,
            req: &AddRouterEntryReq,
        ) -> Result<(), Error> {
            self.hdl.add_router_entry(req)?;
            Ok(())
        }

        fn del_router_entry(
            &self,
            req: &DelRouterEntryReq,
        ) -> Result<DelRouterEntryResp, Error> {
            let resp = self.hdl.del_router_entry(req)?;
            Ok(resp)
        }

        fn allow_cidr(
            &self,
            port_name: &str,
            cidr: IpCidr,
            dir: Direction,
        ) -> Result<(), Error> {
            self.hdl.allow_cidr(port_name, cidr, dir)?;
            Ok(())
        }
    }
}

#[cfg(not(target_os = "illumos"))]
mod noop {
    use super::OpteImpl;
    use super::*;

    pub(crate) struct OpteImplNoop;

    impl OpteImpl for OpteImplNoop {
        fn open() -> Result<Self, Error> {
            Ok(Self)
        }

        fn create_xde(
            &self,
            _name: &str,
            _vpc: oxide_vpc::api::VpcCfg,
            _dhcp: oxide_vpc::api::DhcpCfg,
            _passthrough: bool,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn set_fw_rules(&self, _req: &SetFwRulesReq) -> Result<(), Error> {
            Ok(())
        }

        fn add_router_entry(
            &self,
            _req: &AddRouterEntryReq,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn del_router_entry(
            &self,
            _req: &DelRouterEntryReq,
        ) -> Result<DelRouterEntryResp, Error> {
            Ok(DelRouterEntryResp::Ok)
        }

        fn allow_cidr(
            &self,
            _port_name: &str,
            _cidr: IpCidr,
            _dir: Direction,
        ) -> Result<(), Error> {
            Ok(())
        }
    }
}
