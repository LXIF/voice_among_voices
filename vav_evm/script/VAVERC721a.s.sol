// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "forge-std/Script.sol";
import "../src/VAVERC721a.sol";
import "forge-std/console.sol";

contract DisburseNFTsScript is Script {
    VoiceAmongVoices public nftContract;
    address public disbursementWallet;
    uint256 public disbursementPrivateKey;
    
    // CSV parsing helper
    struct Disbursement {
        uint256 tokenId;
        address recipient;
        string name;
    }

    function setUp() public {
        // Auto-generate a new disbursement wallet for this operation
        disbursementPrivateKey = uint256(keccak256(abi.encodePacked(
            block.timestamp, 
            block.prevrandao, 
            msg.sender
        )));
        disbursementWallet = vm.addr(disbursementPrivateKey);
        
        console.log("Generated disbursement wallet:");
        console.log("Address:", disbursementWallet);
        console.log("Private Key:", vm.toString(disbursementPrivateKey));
        console.log("");
    }

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        disbursementWallet = vm.addr(deployerPrivateKey);
        disbursementPrivateKey = deployerPrivateKey;
        
        // Add a pause for manual ETH transfer
        console.log("=== PAUSE FOR MANUAL ETH TRANSFER ===");
        console.log("Please transfer ETH to your deployment wallet then rerun:");
        console.log("Address:", vm.addr(deployerPrivateKey));
        console.log("Network: Base Mainnet");
        console.log("Estimated needed: 0.0001 ETH");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Step 1: Deploy the NFT contract
        console.log("Deploying VoiceAmongVoices NFT contract...");
        nftContract = new VoiceAmongVoices();
        console.log("Contract deployed at:", address(nftContract));
        
        // Step 2: Mint all 360 tokens to disbursement wallet
        console.log("Minting 360 tokens to disbursement wallet...");
        nftContract.mint{value: 0}(360);
        console.log("Minting complete!");
        
        console.log("Starting token disbursement...");
        Disbursement[] memory disbursements = readDisbursementCSV();
        
        for (uint256 i = 0; i < disbursements.length; i++) {
            Disbursement memory disp = disbursements[i];
            
            console.log("Transferring token", disp.tokenId, "to", disp.recipient);
            
            // Now we can directly call transferFrom since we're broadcasting as disbursement wallet
            nftContract.transferFrom(disbursementWallet, disp.recipient, disp.tokenId);
            
            // Verify transfer
            address owner = nftContract.ownerOf(disp.tokenId);
            require(owner == disp.recipient, "Transfer verification failed");
            console.log("Transfer verified for token", disp.tokenId);
        }
        
        vm.stopBroadcast();
        
        console.log("Deployment and disbursement complete!");
        console.log("Contract address:", address(nftContract));
        console.log("All tokens transferred successfully.");
        
        // Step 4: Final verification
        verifyDisbursements(disbursements);
        
        // Step 5: Output deployment info and wallet details
        console.log("");
        console.log("=== DEPLOYMENT SUMMARY ===");
        console.log("Contract:", address(nftContract));
        console.log("Network: Base Mainnet");
        console.log("Total tokens minted: 360");
        console.log("Total tokens disbursed:", disbursements.length);
        console.log("");
        console.log("=== DISBURSEMENT WALLET INFO ===");
        console.log("Address:", disbursementWallet);
        console.log("Private Key:", vm.toString(disbursementPrivateKey));
        console.log("Note: This wallet was generated for this operation only");
        console.log("");
        console.log("To verify on Basescan, use:");
        console.log("forge verify-contract", address(nftContract), "src/VAVERC721a.sol:VoiceAmongVoices --chain-id 8453 --etherscan-api-key $BASESCAN_API_KEY --compiler-version 0.8.19");
    }
    
    function readDisbursementCSV() internal view returns (Disbursement[] memory) {
        string memory csvContent = vm.readFile("disbursement.csv");
        string[] memory lines = vm.split(csvContent, "\n");
        
        // Skip header row and count valid lines
        uint256 validLines = 0;
        for (uint256 i = 1; i < lines.length; i++) {
            if (bytes(lines[i]).length > 0) {
                validLines++;
            }
        }
        
        Disbursement[] memory disbursements = new Disbursement[](validLines);
        uint256 disbursementIndex = 0;
        
        for (uint256 i = 1; i < lines.length; i++) {
            if (bytes(lines[i]).length > 0) {
                string[] memory parts = vm.split(lines[i], ",");
                require(parts.length >= 3, "Invalid CSV format");
                
                uint256 tokenId = vm.parseUint(parts[0]);
                address recipient = vm.parseAddress(parts[1]);
                string memory name = parts[2];  // Parse the name column
                
                disbursements[disbursementIndex] = Disbursement(tokenId, recipient, name);
                disbursementIndex++;
            }
        }
        
        return disbursements;
    }
    
    function verifyDisbursements(Disbursement[] memory disbursements) internal view {
        console.log("Verifying all disbursements...");
        
        uint256 verifiedCount = 0;
        
        for (uint256 i = 0; i < disbursements.length; i++) {
            Disbursement memory disp = disbursements[i];
            address actualOwner = nftContract.ownerOf(disp.tokenId);
            
            require(actualOwner == disp.recipient, 
                string.concat("Verification failed: Token ", vm.toString(disp.tokenId)));
            
            // Fix: Use string concatenation instead of multiple arguments
            console.log(string.concat("OK Token ", vm.toString(disp.tokenId), " verified to ", vm.toString(disp.recipient), " (", disp.name, ")"));
            verifiedCount++;
        }
        
        console.log("");
        console.log("=== VERIFICATION SUMMARY ===");
        console.log("Total tokens verified:", vm.toString(verifiedCount));
        console.log("Expected tokens:", vm.toString(disbursements.length));
        
        require(verifiedCount == disbursements.length, "Verification count mismatch");
        require(verifiedCount == 360, "Total token count mismatch");
        
        console.log(string.concat("OK All ", vm.toString(verifiedCount), " tokens verified successfully!"));
        console.log("OK Total supply matches expected 360 tokens");
    }
}