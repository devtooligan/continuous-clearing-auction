// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import {Test} from 'forge-std/Test.sol';
import {Strings} from 'openzeppelin-contracts/contracts/utils/Strings.sol';
import {ConstantsLib} from 'src/libraries/ConstantsLib.sol';
import {DemandLib} from 'src/libraries/DemandLib.sol';
import {FixedPoint96} from 'src/libraries/FixedPoint96.sol';
import {ValueX7} from 'src/libraries/ValueX7Lib.sol';

contract DemandLibOracleExampleTest is Test {
    using Strings for uint256;

    string internal oracleBinary;

    function setUp() public {
        oracleBinary = vm.envOr('CCA_ORACLE_BIN', string('audit/rust-oracle-ffi/target/release/cca-oracle'));
    }

    function test_requiredDemandAtPrice_matchesExactOracleProjection() public {
        _assertRequiredDemand(FixedPoint96.Q96 * ConstantsLib.MPS, FixedPoint96.Q96, ConstantsLib.MPS);
        _assertRequiredDemand(FixedPoint96.Q96 * 7, FixedPoint96.Q96 + 3, ConstantsLib.MPS - 1);
        _assertRequiredDemand(21 * FixedPoint96.Q96, 5, 2);
    }

    function test_canClearSupplyAtPrice_matchesExactOracleThreshold() public {
        uint256 remainingSupplyQ96X7 = 21 * FixedPoint96.Q96;
        uint256 priceQ96 = 5;
        uint256 remainingMps = 2;
        uint256 required = DemandLib.requiredDemandAtPrice(ValueX7.wrap(remainingSupplyQ96X7), priceQ96, remainingMps);

        _assertCanClear(required - 1, remainingSupplyQ96X7, priceQ96, remainingMps);
        _assertCanClear(required, remainingSupplyQ96X7, priceQ96, remainingMps);
        _assertCanClear(required + 1, remainingSupplyQ96X7, priceQ96, remainingMps);
    }

    function test_toPriceCeiling_matchesExactOracleProjection() public {
        _assertPriceCeiling(17, 5 * FixedPoint96.Q96, 3);
        _assertPriceCeiling(21, FixedPoint96.Q96 * 7, ConstantsLib.MPS - 1);
        _assertPriceCeiling(type(uint128).max, type(uint128).max - 1, 1);
    }

    function test_currencyRaisedAtPrice_matchesExactOracleProjection() public {
        _assertCurrencyRaisedAtPrice(1000 * FixedPoint96.Q96 * ConstantsLib.MPS, 700, 100, 5, 3, 9);
        _assertCurrencyRaisedAtPrice(
            1000 * FixedPoint96.Q96 * ConstantsLib.MPS,
            1,
            999 * FixedPoint96.Q96,
            FixedPoint96.Q96,
            100,
            ConstantsLib.MPS
        );
    }

    function _assertRequiredDemand(uint256 remainingSupplyQ96X7, uint256 priceQ96, uint256 remainingMps) internal {
        uint256 actual = DemandLib.requiredDemandAtPrice(ValueX7.wrap(remainingSupplyQ96X7), priceQ96, remainingMps);
        string[] memory args = new string[](3);
        args[0] = remainingSupplyQ96X7.toString();
        args[1] = priceQ96.toString();
        args[2] = remainingMps.toString();
        assertEq(actual, _oracle('required-demand-ceil', args), 'required demand');
    }

    function _assertCanClear(uint256 demandQ96, uint256 remainingSupplyQ96X7, uint256 priceQ96, uint256 remainingMps)
        internal
    {
        bool actual = DemandLib.canClearSupplyAtPrice(demandQ96, remainingSupplyQ96X7, priceQ96, remainingMps);
        string[] memory args = new string[](4);
        args[0] = demandQ96.toString();
        args[1] = remainingSupplyQ96X7.toString();
        args[2] = priceQ96.toString();
        args[3] = remainingMps.toString();
        assertEq(actual, _oracle('can-clear-ideal', args) == 1, 'can clear');
    }

    function _assertPriceCeiling(uint256 demandQ96, uint256 remainingSupplyQ96X7, uint256 remainingMps) internal {
        uint256 actual = DemandLib.toPriceCeiling(demandQ96, remainingSupplyQ96X7, remainingMps);
        string[] memory args = new string[](3);
        args[0] = demandQ96.toString();
        args[1] = remainingSupplyQ96X7.toString();
        args[2] = remainingMps.toString();
        assertEq(actual, _oracle('price-ceil', args), 'price ceiling');
    }

    function _assertCurrencyRaisedAtPrice(
        uint256 remainingSupplyQ96X7,
        uint256 demandAtPriceQ96,
        uint256 demandAbovePriceQ96,
        uint256 priceQ96,
        uint256 deltaMps,
        uint256 remainingMps
    ) internal {
        uint256 actual = ValueX7.unwrap(
            DemandLib.currencyRaisedAtPrice(
                ValueX7.wrap(remainingSupplyQ96X7),
                demandAtPriceQ96,
                demandAbovePriceQ96,
                priceQ96,
                deltaMps,
                remainingMps
            )
        );
        string[] memory args = new string[](6);
        args[0] = remainingSupplyQ96X7.toString();
        args[1] = demandAtPriceQ96.toString();
        args[2] = demandAbovePriceQ96.toString();
        args[3] = priceQ96.toString();
        args[4] = deltaMps.toString();
        args[5] = remainingMps.toString();
        assertEq(actual, _oracle('currency-raised-clearing-tick-ceil', args), 'currency raised at price');
    }

    function _oracle(string memory command, string[] memory inputs) internal returns (uint256) {
        string[] memory args = new string[](inputs.length + 2);
        args[0] = oracleBinary;
        args[1] = command;
        for (uint256 i = 0; i < inputs.length; i++) {
            args[i + 2] = inputs[i];
        }
        return abi.decode(vm.ffi(args), (uint256));
    }
}
