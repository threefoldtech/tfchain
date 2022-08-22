/**
 * All module prefixes except those mentioned in the skippedModulesPrefix will be added to this by the script.
 * If you want to add any past module or part of a skipped module, add the prefix here manually.
 *
 * Any storage value’s hex can be logged via console.log(api.query.<module>.<call>.key([...opt params])),
 * e.g. console.log(api.query.timestamp.now.key()).
 *
 * If you want a map/doublemap key prefix, you can do it via .keyPrefix(),
 * e.g. console.log(api.query.system.account.keyPrefix()).
 *
 * For module hashing, do it via xxhashAsHex,
 * e.g. console.log(xxhashAsHex('System', 128)).
 */

const fs = require("fs");
const path = require("path");
const chalk = require("chalk");
require("dotenv").config();
const { ApiPromise } = require("@polkadot/api");
const { HttpProvider } = require("@polkadot/rpc-provider");
const { xxhashAsHex } = require("@polkadot/util-crypto");
const schemaPath = path.join(__dirname, "data", "schema.json");

// Using http endpoint since substrate's Ws endpoint has a size limit.
const provider = new HttpProvider("http://185.206.122.126:9933");

async function main() {
  let api;
  console.log(
    chalk.green(
      "We are intentionally using the HTTP endpoint. If you see any warnings about that, please ignore them."
    )
  );
  if (!fs.existsSync(schemaPath)) {
    console.log(chalk.yellow("Custom Schema missing, using default schema."));
    api = await ApiPromise.create({ provider });
  } else {
    const { types, rpc } = JSON.parse(fs.readFileSync(schemaPath, "utf8"));
    api = await ApiPromise.create({
      provider,
      types,
    });
    api.connect();
  }

  console.log(xxhashAsHex("Session", 128));
  let key = await api.query.session.validators.key();
  console.log(key);
}

main();
