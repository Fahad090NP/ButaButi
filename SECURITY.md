# Security Policy

## Supported Versions

We actively support the following versions of Butabuti with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of Butabuti seriously. If you discover a security vulnerability, please follow these guidelines:

### Where to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security issues via one of these methods:

1. **Email**: Send details to the project maintainers (contact information in [Cargo.toml](Cargo.toml))
2. **GitHub Security Advisories**: Use the [private vulnerability reporting feature](https://github.com/Fahad090NP/butabuti/security/advisories/new) (recommended)

### What to Include

When reporting a vulnerability, please include:

- **Description**: Clear description of the vulnerability
- **Impact**: What an attacker could achieve
- **Reproduction**: Step-by-step instructions to reproduce the issue
- **Affected Versions**: Which versions are affected
- **Environment**: OS, Rust version, dependency versions (if relevant)
- **Proof of Concept**: Code or files demonstrating the issue (if applicable)
- **Suggested Fix**: If you have ideas for a fix (optional)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity (critical issues prioritized)
- **Disclosure**: Coordinated disclosure after fix is available

### Security Update Process

1. **Acknowledgment**: We confirm receipt and begin investigation
2. **Assessment**: We evaluate severity and impact
3. **Fix Development**: We develop and test a fix
4. **Release**: We publish a security patch
5. **Disclosure**: We publicly disclose the vulnerability after users have had time to update

### Severity Levels

- **Critical**: Immediate risk of data loss, remote code execution, or compromise
- **High**: Significant security risk affecting most users
- **Medium**: Moderate security risk with limited impact
- **Low**: Minor security concern or theoretical vulnerability

## Security Best Practices

When using Butabuti in your applications:

### Input Validation

- **Untrusted Files**: Always validate embroidery files from untrusted sources
- **File Size Limits**: Implement reasonable file size limits to prevent DoS
- **Format Detection**: Use the built-in format detector before parsing

```rust
use butabuti::formats::io::detector::detect_format;

// Validate file size
if file_size > MAX_FILE_SIZE {
    return Err("File too large");
}

// Detect format before parsing
let format = detect_format(&mut file)?;
if !is_allowed_format(&format) {
    return Err("Unsupported format");
}
```

### Resource Limits

- **Memory**: Large patterns can consume significant memory
- **Processing Time**: Set timeouts for pattern processing
- **Stitch Count**: Validate reasonable stitch counts (e.g., < 1,000,000)

```rust
// Check stitch count after reading
if pattern.stitches().len() > MAX_STITCHES {
    return Err("Pattern too complex");
}
```

### Error Handling

- **Don't Panic**: The library returns `Result` types - handle errors gracefully
- **Sanitize Errors**: Don't expose internal paths or details in user-facing errors
- **Logging**: Log security-relevant events (unusual file formats, excessive resource use)

### File Permissions

- **Temporary Files**: Use secure temporary directories with restricted permissions
- **Output Files**: Set appropriate file permissions when writing files
- **Directory Traversal**: Validate file paths to prevent directory traversal attacks

## Known Security Considerations

### File Format Parsing

- **Binary Formats**: Parsing untrusted binary files has inherent risks
- **Memory Safety**: Rust's memory safety helps, but logic bugs can still occur
- **Fuzz Testing**: We use property-based testing and fuzzing (see `tests/fuzz_formats.rs`)

### Dependencies

- **Regular Updates**: We regularly update dependencies for security patches
- **Dependency Audit**: Run `cargo audit` to check for known vulnerabilities
- **Minimal Dependencies**: We minimize dependencies to reduce attack surface

### WASM Security

- **Sandboxing**: WASM runs in a browser sandbox (limited file system access)
- **Memory Limits**: Browser imposes memory limits on WASM
- **Origin Policy**: Serve WASM from the same origin to prevent CORS issues

## Security Changelog

### Version 0.1.0 (Initial Release)

- Implemented error handling for all format readers/writers
- Added format detection to prevent incorrect parsing
- Added input validation for metadata fields
- Included fuzz testing infrastructure
- No known security vulnerabilities at release

## Additional Resources

- [Rust Security Working Group](https://www.rust-lang.org/governance/wgs/wg-security)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE - Common Weakness Enumeration](https://cwe.mitre.org/)

## Security Contact

For security issues that require private disclosure, please contact the project maintainers through the GitHub Security Advisories feature or via email (see [Cargo.toml](Cargo.toml) for contact information).

**Thank you for helping keep Butabuti secure!**
