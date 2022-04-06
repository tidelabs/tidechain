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

use crate::mock::{new_test_ext, AssetRegistry};

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    // make sure we have TIFI + 2 custom currencies registered
    assert_eq!(
      AssetRegistry::get_assets()
        .expect("Unable to get results")
        .len(),
      3
    )
  });
}
