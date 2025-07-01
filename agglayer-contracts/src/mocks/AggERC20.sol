// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Burnable} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract AggERC20 is ERC20, ERC20Burnable, Ownable, ERC20Permit {
    constructor(address recipient, address initialOwner, uint256 mintAmount)
        ERC20("AggERC20", "AGGERC20")
        Ownable(initialOwner)
        ERC20Permit("AggERC20")
    {
        _mint(recipient, mintAmount * 10 ** decimals());
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
