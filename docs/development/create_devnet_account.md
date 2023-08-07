# Creating And Funding An Account on TFChain Development network

## the TF dashboard flow:

**Caution**: This option allows you to create and store your account locally in your browser. However, you need strong backup policy as this method is not very secure and you may lose your account if you clear your browser data.

- Visit https://dashboard.dev.grid.tf and click on the `Generate Account` button.
- Read the ThreeFold Grid USER/FARMER TERMS AND CONDITIONS and click on the `Accept Terms and Conditions` button.
- You will see a 12-word mnemonic phrase (click on the eye icon to toggle the visibility). This is your private key, so write it down and keep it safe. You also need to choose a password that will encrypt the mnemonic in your browser local storage.
- After you have saved the mnemonic securely and confirmed your password, click on the `Connect` button. You can use the password on the same machine and browser to access your wallet without entering the mnemonic. Now your account was created and activated on the TFchain development network.

## Alternative flows:
Note: The methods mentioned below are outdated and no longer work.
### Polkadot-JS UI flow:

**caution**: This option allows you to create and store your account locally in your browser. However, this is not very secure and you may lose your account if you clear your browser data. To use this option, you need to:

- Step 1: Browse to the Polkadot JS Apps UI

You can use these link to access it:
- Using a public node: https://polkadot.js.org/apps/?rpc=wss://tfchain.dev.grid.tf/ws#/
- Using a private node: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944/ws#/


- Step 2: Enable in-browser Account Creation

To enable account creation on Polkadot-JS UI, navigate to the Settings tab. In the account options, choose `allow local in-browser account storage` and click Save.

Note: This option is provided for advanced users with strong backup policies. It is recommended that you store all keys externally to the in-page browser local storage, either on browser extensions, signers operating via QR codes or hardware devices. 

- step 3: Navigate to the "Accounts" page

Should look something like this:

![image](https://user-images.githubusercontent.com/13766992/130954090-c34193eb-0864-4f6a-aa49-7ce66b6d72fb.png)

- Step 4: Create New Account

- Click "Add Account"

![image](https://user-images.githubusercontent.com/13766992/130955887-b6e87bc4-64d5-49ff-b6ac-fa6ac2ebc90d.png)

- Once pressed you'll see a 12-word mnemonic phrase. Save it safely (Make sure to save your mnemonic phrase now, as there is no way to view it after the account is created) and check the box saying you have done so and press next.
- Enter a name for your account in the Name field. This is just a label for your convenience and does not affect the functionality of the account.
- Enter a password for your account in the Password and Confirm Password fields. This is used to encrypt your private key locally and protect your account from unauthorized access.
- Optionally, you can choose a different derivation path for your account in the Advanced creation options section. This is an advanced feature that allows you to generate different accounts from the same seed phrase. You can leave it as default if you are not familiar with it.
- Optionally, click on the Advanced creation options section to expand it.
    - You will see a drop-down menu labeled Key type. This is where you can select the type of cryptographic algorithm for your account’s key pair.
    - The default option is sr25519, which is a Schnorr signature scheme that offers high performance and security. You can also choose ed25519, which is an Edwards curve signature scheme that is widely used and compatible with many platforms. Both options support Substrate-based networks and Polkadot.

- After you select your preferred key type, you can proceed to click on the Save button and create your account.

- Step 5: Fund your account

- On the same page, on the left top, hover over `Account` button and click on `Transfer`.
- First select account `Alice` and secondly select your newly created account from the list.
- Send any amount to your account.

### Polkadot Browser extension flow:

This option allows you to create and store your account in a separate browser extension that can be accessed from any website. This is more secure and convenient than the in-browser storage option. To use this option, you need to:

- step 1: Install the Polkadot JS Extension for your browser (Chrome, Firefox, or Brave).
- step 2: Click on the extension icon on your browser toolbar and then click on the + button and select `Create new account` to create a new account.
- step 3: You will see your secret seed phrase (Generated 12-WORD mnemonic seed), which you must write down and keep safe. mark the box beside `I have stored my mnemonic seed safely` and click on the `Next step` button.
- step 4: Follow the instructions to generate a new account. Leave the network as it is, and choose a name, and a password. 
- step 5: Click on the `Add the account with the generated seed` button to store your account in the extension. You will see your account address and icon on the extension popup. You can also export your account as a JSON file for backup purposes.


Once you have created and stored your account using either option, you can use it to interact with TFchain or any Substrate-based. You can also create multiple accounts for different purposes using different derivation paths.
