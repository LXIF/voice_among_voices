// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "../lib/ERC721A";

contract VoiceAmongVoices is ERC721A {
    constructor() ERC721A("VoiceAmongVoices", "VAV") {}

    uint256 public constant MAX_SUPPLY = 360;
    string constant BASE_URI = "https://anpkp-byaaa-aaaad-aamdq-cai.icp0.io/";

    function mint(uint256 quantity) external payable {
        require(_totalMinted() + quantity <= MAX_SUPPLY, "Exceeds max supply");
        _mint(msg.sender, quantity);
    }

    function _baseURI() internal view virtual override returns (string memory) {
        return BASE_URI;
    }

    function tokenOfOwnerByIndex(address owner, uint256 index) public view returns (uint256) {
        uint256 tokenCount = balanceOf(owner);
        require(index < tokenCount, "tokenIndex out of bounds");
        uint256 counter = 0;
        uint256 tokenId = 0;
        for (uint256 i = 0; i < MAX_SUPPLY; ++i) {
            if(ownerOf(i) == owner) {
                if(counter == index) {
                    tokenId = i;
                    break;
                }
                counter++;
            }
        }
        return tokenId;
    }
}