# Security Checklist

## Secret Protection
- [ ] Private keys never appear in request bodies
- [ ] Secret-leak detection scans all outgoing data
- [ ] Keyfile stored with 0600 permissions
- [ ] Password never logged or exposed

## Authentication
- [ ] All API calls include auth headers
- [ ] Timestamps within ±5 min drift tolerance
- [ ] EIP-712 signatures verified on-chain

## Input Validation
- [ ] Prediction text sanitized (no control characters)
- [ ] Backend URL validated (no SSRF vectors)
- [ ] Private key format validated before use
