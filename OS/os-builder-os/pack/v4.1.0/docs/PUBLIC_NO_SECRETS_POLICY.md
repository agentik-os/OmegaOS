# Public No-Secrets Policy

Builder {OS} packages are public-by-default. Generated operating systems MUST NOT contain API keys, provider credentials, tokens, secrets, private certificates, `.env` credential boilerplate, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `sk-` secrets, signing keys, or instructions that make a credential mandatory for core use.

## Rules

1. Core OS logic must run as documentation, prompts, workflows, schemas, agents, commands, and local/public components without a secret.
2. Integrations are optional adapters only. Describe capabilities and interfaces, never embed credentials.
3. If an external service requires authentication, document it abstractly as an optional connector supplied by the host runtime.
4. Public release validation fails if credential material or credential boilerplate is detected.
5. Checksums are integrity metadata and are allowed. They are not secret keys.
