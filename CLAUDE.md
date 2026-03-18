# Coding Conventions

## Stack
- Node.js with TypeScript 5.3
- ethers.js v6 for blockchain interaction
- chalk v4 for CLI output
- dotenv for environment variables

## Naming Conventions
- Functions: camelCase
- Constants: UPPER_SNAKE_CASE
- Files: kebab-case.ts
- Types/Interfaces: PascalCase

## Import Conventions
- Use relative imports within src/
- Keep ethers as external dependency

## Error Handling
- All API calls use try-catch with descriptive error messages
- Secret-leak detection runs before any outgoing request
- Auth headers attached to every backend request

## Testing
- Manual testing via battle scripts
- `npm run build` must succeed

## Build
- TypeScript compiles to /dist/ directory
- Target: ES2020, Module: CommonJS
- Entry point: claw-cli.ts
