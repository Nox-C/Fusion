// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IFlashLoanProvider {
    function flashLoan(
        address receiver,
        address token,
        uint256 amount,
        bytes calldata data
    ) external;
}

interface IERC20 {
    function transfer(address to, uint256 value) external returns (bool);
    function balanceOf(address owner) external view returns (uint256);
    function approve(address spender, uint256 value) external returns (bool);
}

interface IDEXRouter {
    function swapExactTokensForTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external returns (uint[] memory amounts);
}

contract ArbitrageExecutor {
    address public owner;
    address public profitRecipient;

    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }

    event ArbitrageExecuted(address indexed initiator, uint profit);

    constructor(address _profitRecipient) {
        owner = msg.sender;
        profitRecipient = _profitRecipient;
    }

    function setProfitRecipient(address _recipient) external onlyOwner {
        profitRecipient = _recipient;
    }

    // Main entry: called by owner/backend with all parameters
    function executeArbitrage(
        address flashLoanProvider,
        address loanToken,
        uint256 loanAmount,
        address[] calldata routers,
        address[][] calldata swapPaths,
        uint256[] calldata amountsIn,
        uint256[] calldata amountsOutMin
    ) external onlyOwner {
        require(
            routers.length == swapPaths.length &&
            routers.length == amountsIn.length &&
            routers.length == amountsOutMin.length,
            "Input array mismatch"
        );
        // Initiate flashloan; callback will execute swaps
        bytes memory data = abi.encode(
            routers, swapPaths, amountsIn, amountsOutMin
        );
        IFlashLoanProvider(flashLoanProvider).flashLoan(
            address(this), loanToken, loanAmount, data
        );
    }

    // Called by flashloan provider
    function onFlashLoan(
        address initiator,
        address token,
        uint256 amount,
        uint256 fee,
        bytes calldata data
    ) external {
        require(msg.sender == tx.origin || msg.sender == owner, "Not authorized");
        (
            address[] memory routers,
            address[][] memory swapPaths,
            uint256[] memory amountsIn,
            uint256[] memory amountsOutMin
        ) = abi.decode(data, (address[], address[][], uint256[], uint256[]));
        // Approve and perform swaps
        address currentToken = token;
        uint256 currentAmount = amount;
        for (uint i = 0; i < routers.length; i++) {
            IERC20(currentToken).approve(routers[i], amountsIn[i]);
            uint[] memory amounts = IDEXRouter(routers[i]).swapExactTokensForTokens(
                amountsIn[i],
                amountsOutMin[i],
                swapPaths[i],
                address(this),
                block.timestamp
            );
            currentToken = swapPaths[i][swapPaths[i].length - 1];
            currentAmount = amounts[amounts.length - 1];
        }
        // Repay flashloan
        uint256 totalOwed = amount + fee;
        require(currentAmount >= totalOwed, "No profit");
        IERC20(currentToken).transfer(msg.sender, totalOwed);
        // Send profit to recipient
        uint256 profit = currentAmount - totalOwed;
        if (profit > 0) {
            IERC20(currentToken).transfer(profitRecipient, profit);
        }
        emit ArbitrageExecuted(initiator, profit);
    }
}
