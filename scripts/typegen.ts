import codegen from "@cosmwasm/ts-codegen";

async function generateSDK() {
  try {
    await codegen({
      contracts: [
        {
          name: "coreum-fun",
          dir: "./src/schema",
        },

      ],
      outPath: "./typegen",
      options: {
        bundle: {
          bundleFile: "index.ts",
          scope: "contracts",
        },
        types: {
          enabled: true,
        },
        client: {
          enabled: true,
        },
        reactQuery: {
          enabled: true,
          optionalClient: true,
          version: "v4",
          mutations: true,
          queryKeys: true,
          queryFactory: true,
        },
        recoil: {
          enabled: false,
        },
        messageComposer: {
          enabled: false,
        },
        messageBuilder: {
          enabled: false,
        },
        useContractsHook: {
          enabled: true,
        },
      },
    });

    console.log("✨ TypeScript SDK generation complete!");
  } catch (error) {
    console.error("❌ SDK generation failed:", error);
    process.exit(1);
  }
}

generateSDK();
