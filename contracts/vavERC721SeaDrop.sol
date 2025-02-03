// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

import {ERC721SeaDrop} from "./ERC721SeaDrop.sol";
import {ERC2981} from "@openzeppelin/contracts/token/common/ERC2981.sol";

contract VavERC721 is ERC721SeaDrop, ERC2981 {
    constructor(
        string memory name,
        string memory symbol,
        address[] memory allowedSeaDrop,
        address royaltyReceiver,
        uint96 royaltyFeeNumerator
    ) ERC721SeaDrop(name, symbol, allowedSeaDrop) {
        // Set royalty info
        _setDefaultRoyalty(royaltyReceiver, royaltyFeeNumerator);

        // Batch mint tokens 1-359 to contract owner
        for (uint256 i = 1; i <= 359; i++) {
            _safeMint(msg.sender, i);
        }
    }

    // Override required by Solidity
    function supportsInterface(
        bytes4 interfaceId
    ) public view virtual override(ERC721SeaDrop, ERC2981) returns (bool) {
        return
            super.supportsInterface(interfaceId) ||
            interfaceId == type(IERC2981).interfaceId;
    }

    // Override to enforce max supply of 359
    function maxSupply() public pure override returns (uint256) {
        return 359;
    }
}
