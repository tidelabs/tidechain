// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
  mock::{new_test_ext, Event as MockEvent, Origin, Security, System},
  pallet::*,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use tidefi_primitives::StatusCode;

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {});
}

#[test]
pub fn set_migration_operational_status_works() {
  new_test_ext().execute_with(|| {});
}

mod set_status {
  use super::*;

  #[test]
  fn succeeds() {
    new_test_ext().execute_with(|| {
      assert_eq!(Security::status(), StatusCode::Running);
      assert_ok!(Security::set_status(
        Origin::root(),
        StatusCode::Maintenance
      ));
      assert_eq!(Security::status(), StatusCode::Maintenance);
      // System::assert_has_event(MockEvent::Security(Event::StatusChanged(
      //   StatusCode::Maintenance,
      // )));
    });
  }

  #[test]
  fn fails_when_signer_is_not_root() {
    new_test_ext().execute_with(|| {
      assert_noop!(
        Security::set_status(Origin::signed(1.into()), StatusCode::Maintenance),
        BadOrigin
      );
    });
  }
}
