module.exports = {
  ci: {
    collect: {
      startServerCommand:
        "pnpm website:preview --host 127.0.0.1 --port 4173",
      startServerReadyPattern: "Local:",
      url: [
        "http://127.0.0.1:4173/pluck/",
        "http://127.0.0.1:4173/pluck/zh-CN/",
      ],
      numberOfRuns: 1,
      settings: { preset: "desktop" },
    },
    assert: {
      assertions: {
        "categories:performance": ["error", { minScore: 0.95 }],
        "categories:accessibility": ["error", { minScore: 0.95 }],
        "categories:best-practices": ["error", { minScore: 0.95 }],
        "categories:seo": ["error", { minScore: 1 }],
      },
    },
    upload: {
      target: "filesystem",
      outputDir: "website/.lighthouse/desktop",
    },
  },
};
