# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in HwpBridge, please report it responsibly.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please email: **parkdavid31@gmail.com**

Include the following information:

1. Description of the vulnerability
2. Steps to reproduce
3. Potential impact
4. Any suggested fixes (optional)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Target**: Depends on severity

### Severity Levels

| Severity | Description | Target Resolution |
|----------|-------------|-------------------|
| Critical | Remote code execution, data exfiltration | 24-48 hours |
| High | Significant security bypass | 7 days |
| Medium | Limited security impact | 30 days |
| Low | Minimal security impact | 90 days |

### What to Expect

1. We will acknowledge your report promptly
2. We will investigate and assess the vulnerability
3. We will work on a fix and coordinate disclosure
4. We will credit you in the security advisory (if desired)

### Scope

This security policy applies to:

- `hwp-core` - Core parsing library
- `hwp-types` - Type definitions
- `hwp-cli` - Command-line interface
- `hwp-mcp` - MCP server

### Out of Scope

- Vulnerabilities in dependencies (report to upstream)
- Social engineering attacks
- Physical attacks
- Denial of service attacks

## Security Best Practices

When using HwpBridge:

1. **Validate Input**: Always validate HWP files before processing
2. **Sandbox Execution**: Run in isolated environments for untrusted files
3. **Keep Updated**: Use the latest version for security fixes
4. **Monitor Resources**: Set appropriate limits for memory and CPU

## Contact

Security issues: parkdavid31@gmail.com

For non-security issues, please use GitHub Issues.
