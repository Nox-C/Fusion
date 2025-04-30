// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../src/ArbitrageExecutor.sol";

contract DeployArbitrageExecutor {
    function deploy(address profitRecipient) public returns (ArbitrageExecutor) {
        return new ArbitrageExecutor(profitRecipient);
    }
}
