# Running the Ethereum Interaction Program in ICP

This guide will walk you through setting up your development environment and running a ICP canister program designed for interacting with the Ethereum blockchain, specifically using the Sepolia testnet. 

## Prerequisites

Before you begin, ensure you have a basic understanding of Rust programming and some familiarity with blockchain concepts, particularly Ethereum.

## Step 1: Setting Up the Rust Development Environment

1. **Install Rust CDK**: Follow the instructions provided in the [Rust Quickstart Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/quickstart) to set up your Rust development environment. This guide will help you install Rust and its package manager, Cargo.

2. **Clone the Repository**: Clone the repository containing the Rust program you want to run.



3. **Navigate to Your Project**: Use the command line to navigate to your project directory. For example:
   ```bash
   sudo dfx start --clean
   ```
4. **Start the Internet Computer Replica**: Start the Internet Computer replica by running the following command:
   ```
   sudo dfx start --clean
   ```
5. ## Step 2: Setting Up an Ethereum Node Provider

If the node provider URL in the example code is not working, you'll need to set up your own node provider. One easy way to do this is through Alchemy.

- **Create an Alchemy Account**: Go to [Alchemy's website](https://www.alchemy.com/) and sign up for an account.

- **Create a New App**: Once logged in, create a new app. Choose the Sepolia testnet as your network.

- **Get the HTTP URL**: After creating your app, Alchemy will provide you with an HTTP URL. Replace the `URL` constant in your Rust code with this new URL.

6 Deploy canister locally
```
sudo dfx deploy
```
7. Get Sepolia Testnet Faucets
   - Go to deployed canister candid frontend url
   - Click on this function get_address to get your address
     ![image](https://github.com/dipanshuhappy/basic_eth_icp/assets/58115782/f4d1537c-6642-4072-b874-6800e4e281c1)
   - Go to this website https://sepoliafaucet.com/
   - And paste in the address which you got from that function

   

