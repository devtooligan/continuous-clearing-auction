// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import {Test} from 'forge-std/Test.sol';
import {Strings} from 'openzeppelin-contracts/contracts/utils/Strings.sol';
import {Bid} from 'src/libraries/BidLib.sol';
import {CheckpointAccountingLib} from 'src/libraries/CheckpointAccountingLib.sol';
import {Checkpoint} from 'src/libraries/CheckpointLib.sol';
import {ConstantsLib} from 'src/libraries/ConstantsLib.sol';
import {ValueX7} from 'src/libraries/ValueX7Lib.sol';

contract BidAccountingOracleHarness {
    function fullyFilled(
        uint256 amountQ96,
        uint256 maxPrice,
        uint256 cumulativeMpsPerPriceDelta,
        uint24 cumulativeMpsDelta,
        uint24 mpsRemaining
    ) external pure returns (uint256 tokensFilled, uint256 currencySpentQ96) {
        Bid memory bid = _bid(amountQ96, maxPrice, mpsRemaining);
        Checkpoint memory startCheckpoint = Checkpoint({
            clearingPrice: 0,
            currencyRaisedAtClearingPriceQ96X7: ValueX7.wrap(0),
            cumulativeMpsPerPrice: 0,
            cumulativeMps: bid.startCumulativeMps,
            prev: 0,
            next: 0
        });
        Checkpoint memory upperCheckpoint = Checkpoint({
            clearingPrice: 0,
            currencyRaisedAtClearingPriceQ96X7: ValueX7.wrap(0),
            cumulativeMpsPerPrice: cumulativeMpsPerPriceDelta,
            cumulativeMps: bid.startCumulativeMps + cumulativeMpsDelta,
            prev: 0,
            next: 0
        });
        return CheckpointAccountingLib.accountFullyFilledCheckpoints(upperCheckpoint, startCheckpoint, bid);
    }

    function partiallyFilled(
        uint256 amountQ96,
        uint256 maxPrice,
        uint256 tickDemandQ96,
        uint256 currencyRaisedAtClearingQ96X7,
        uint24 mpsRemaining
    ) external pure returns (uint256 tokensFilled, uint256 currencySpentQ96) {
        return CheckpointAccountingLib.accountPartiallyFilledCheckpoints(
            _bid(amountQ96, maxPrice, mpsRemaining), tickDemandQ96, ValueX7.wrap(currencyRaisedAtClearingQ96X7)
        );
    }

    function _bid(uint256 amountQ96, uint256 maxPrice, uint24 mpsRemaining) internal pure returns (Bid memory) {
        return Bid({
            startBlock: 1,
            startCumulativeMps: uint24(ConstantsLib.MPS - mpsRemaining),
            exitedBlock: 0,
            maxPrice: maxPrice,
            owner: address(1),
            amountQ96: amountQ96,
            tokensFilled: 0
        });
    }
}

contract BidAccountingOracleExampleTest is Test {
    using Strings for uint256;

    string internal oracleBinary;
    BidAccountingOracleHarness internal harness;

    function setUp() public {
        oracleBinary = vm.envOr('CCA_ORACLE_BIN', string('audit/rust-oracle-ffi/target/release/cca-oracle'));
        harness = new BidAccountingOracleHarness();
    }

    function test_fullyFilledBidAccounting_matchesOracleProjection() public {
        uint256 amountQ96 = 10_001;
        uint24 cumulativeMpsDelta = 333;
        uint24 mpsRemaining = 997;
        uint256 cumulativeMpsPerPriceDelta = (uint256(1) << 192) * 11 + 123;

        (uint256 tokensFilled, uint256 currencySpentQ96) =
            harness.fullyFilled(amountQ96, 5, cumulativeMpsPerPriceDelta, cumulativeMpsDelta, mpsRemaining);

        assertEq(currencySpentQ96, _fullFillCurrency(amountQ96, cumulativeMpsDelta, mpsRemaining), 'currency spent');
        assertEq(tokensFilled, _fullFillTokens(amountQ96, cumulativeMpsPerPriceDelta, mpsRemaining), 'tokens filled');
    }

    function test_partiallyFilledBidAccounting_matchesOracleProjection() public {
        uint256 amountQ96 = 10_001;
        uint256 maxPriceQ96 = 7;
        uint256 tickDemandQ96 = 12_345;
        uint256 currencyRaisedAtClearingQ96X7 = 98_765;
        uint24 mpsRemaining = 997;

        (uint256 tokensFilled, uint256 currencySpentQ96) =
            harness.partiallyFilled(amountQ96, maxPriceQ96, tickDemandQ96, currencyRaisedAtClearingQ96X7, mpsRemaining);

        assertEq(
            currencySpentQ96,
            _partialFillCurrency(amountQ96, tickDemandQ96, currencyRaisedAtClearingQ96X7, mpsRemaining),
            'currency spent'
        );
        assertEq(
            tokensFilled,
            _partialFillTokens(amountQ96, maxPriceQ96, tickDemandQ96, currencyRaisedAtClearingQ96X7, mpsRemaining),
            'tokens filled'
        );
    }

    function _fullFillCurrency(uint256 amountQ96, uint24 cumulativeMpsDelta, uint24 mpsRemaining)
        internal
        returns (uint256)
    {
        string[] memory args = new string[](3);
        args[0] = amountQ96.toString();
        args[1] = uint256(cumulativeMpsDelta).toString();
        args[2] = uint256(mpsRemaining).toString();
        return _oracle('full-fill-currency-ceil', args);
    }

    function _fullFillTokens(uint256 amountQ96, uint256 cumulativeMpsPerPriceDelta, uint24 mpsRemaining)
        internal
        returns (uint256)
    {
        string[] memory args = new string[](3);
        args[0] = amountQ96.toString();
        args[1] = cumulativeMpsPerPriceDelta.toString();
        args[2] = uint256(mpsRemaining).toString();
        return _oracle('full-fill-tokens-floor', args);
    }

    function _partialFillCurrency(
        uint256 amountQ96,
        uint256 tickDemandQ96,
        uint256 currencyRaisedAtClearingQ96X7,
        uint24 mpsRemaining
    ) internal returns (uint256) {
        string[] memory args = new string[](4);
        args[0] = amountQ96.toString();
        args[1] = tickDemandQ96.toString();
        args[2] = currencyRaisedAtClearingQ96X7.toString();
        args[3] = uint256(mpsRemaining).toString();
        return _oracle('partial-fill-currency-ceil', args);
    }

    function _partialFillTokens(
        uint256 amountQ96,
        uint256 maxPriceQ96,
        uint256 tickDemandQ96,
        uint256 currencyRaisedAtClearingQ96X7,
        uint24 mpsRemaining
    ) internal returns (uint256) {
        string[] memory args = new string[](5);
        args[0] = amountQ96.toString();
        args[1] = maxPriceQ96.toString();
        args[2] = tickDemandQ96.toString();
        args[3] = currencyRaisedAtClearingQ96X7.toString();
        args[4] = uint256(mpsRemaining).toString();
        return _oracle('partial-fill-tokens-floor', args);
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
