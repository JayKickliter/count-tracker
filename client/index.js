// args:
// 0: program ID
// [1]: counter ID
// [2]: auth pubKey

const {
    Connection,
    sendAndConfirmTransaction,
    Keypair,
    Transaction,
    SystemProgram,
    PublicKey,
    TransactionInstruction,
} = require("@solana/web3.js");
const Base58 = require('base58');
const FS = require('fs');
const BN = require("bn.js");
const {readFile} = require('mz/fs');

const main = async () => {
    var args = process.argv.slice(2);
    const programId = new PublicKey(args[0]);

    const connection = new Connection("http://localhost:8899");
    let feePayer = new Keypair();

    if (args.length > 2) {
        let secretKeyString = await readFile(args[2], {encoding: 'utf8'});
        console.log("Loaded Keypair from ", args[2]);
        const secretKey = Uint8Array.from(Object.values(JSON.parse(secretKeyString)));
        feePayer = Keypair.fromSecretKey(secretKey);
    } else {
        FS.writeFile(feePayer.publicKey.toBase58(), JSON.stringify(Uint8Array.from(feePayer.secretKey)), (err) => {
            if (err)
                console.log(err);
            else {
                console.log("File written successfully\n");
            }
        });
    }

    console.log("feepayer pubKey:", feePayer.publicKey.toBase58());
    console.log("Requesting Airdrop of 1 SOL...");
    await connection.requestAirdrop(feePayer.publicKey, 2e9);
    console.log("Airdrop received");

    const counterAccount = new Keypair();
    let counterKey = counterAccount.publicKey;
    let tx = new Transaction();
    let signers = [feePayer];
    if (args.length > 1) {
        console.log("Found address");
        counterKey = new PublicKey(args[1]);
    } else {
        console.log("Generating new counter address");
        let createIx = SystemProgram.createAccount({
            fromPubkey: feePayer.publicKey,
            newAccountPubkey: counterKey,
            /** Amount of lamports to transfer to the created account */
            lamports: await connection.getMinimumBalanceForRentExemption(36),
            /** Amount of space in bytes to allocate to the created account */
            space: 36,
            /** Public key of the program to assign as the owner of the created account */
            programId: programId,
        });
        signers.push(counterAccount);
        tx.add(createIx);
    }

    let instr_dat = Buffer.from("");
    // if (args.length > 2) {
    //     let toIncrement = parseInt(args[2], 10);
    //     console.log("toIncrement: ", toIncrement);
    //     instr_dat = Buffer.from("    ");
    //     instr_dat.writeInt32LE(toIncrement, 0);
    //     console.log("Incrementing by: ", instr_dat.readUInt32LE());
    // }

    let counterIx = new TransactionInstruction({
        keys: [
            {
                pubkey: counterKey,
                isSigner: false,
                isWritable: true,
            },
            {
                pubkey: feePayer.publicKey,
                isSigner: true,
                isWritable: false,
            },
        ],
        programId: programId,
        data: instr_dat,
    });
    console.log("couinterIx:", counterIx);
    tx.add(counterIx);

    let txid = await sendAndConfirmTransaction(connection, tx, signers, {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        commitment: "confirmed",
    });

    let data = (await connection.getAccountInfo(counterKey, "confirmed")).data;
    console.log("New counter value: ", data.readUInt32LE(32));
    console.log("Counter Key:", counterKey.toBase58());
};

main()
    .then(() => {
        console.log("Success");
    })
    .catch((e) => {
        console.error(e);
    });
