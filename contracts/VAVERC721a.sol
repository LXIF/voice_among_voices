// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "https://github.com/chiru-labs/ERC721A/contracts/ERC721A.sol";

contract VoiceAmongVoices is ERC721A {
    constructor() ERC721A("VoiceAmongVoices_test", "VAVT") {}

    uint256 public constant MAX_SUPPLY = 360;
    string constant BASE_URI = "https://anpkp-byaaa-aaaad-aamdq-cai.icp0.io/";

    function mint(uint256 quantity) external payable {
        require(_totalMinted() + quantity <= MAX_SUPPLY, "Exceeds max supply");
        _mint(msg.sender, quantity);
    }

    function _baseURI() internal view virtual override returns (string memory) {
        return BASE_URI;
    }

    function tokenOfOwnerByIndex(address owner) public view returns (uint256[] memory) {
        uint256 tokenCount = balanceOf(owner);
        uint256[] memory tokenIds = new uint256[](tokenCount);
        uint256 counter = 0;
        for (uint256 i = 0; i < MAX_SUPPLY; ++i) {
            if(ownerOf(i) == owner) {
                tokenIds[counter] = i;
                counter++;
            }
        }
        return tokenIds;
    }
}