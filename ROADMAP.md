# Butabuti Roadmap

This document outlines the planned features and improvements for Butabuti. For detailed task tracking, see [TODOS.md](TODOS.md).

## Vision

**Mission**: Provide the most comprehensive, performant, and reliable embroidery file manipulation library in the Rust ecosystem.

**Core Principles**:

- **Full Format Support**: Only include formats with complete read/write support
- **Memory Safety**: Leverage Rust's guarantees for production reliability
- **Performance**: Optimize for speed without sacrificing correctness
- **Developer Experience**: Make embroidery programming accessible and enjoyable

---

## Release Timeline

### Version 0.1.0 (Current - Q1 2024) ✅

**Status**: Initial Release  
**Theme**: Foundation & Core Functionality

**Delivered:**

- ✅ 15 bidirectional embroidery formats (DST, PES, JEF, VP3, EXP, etc.)
- ✅ Core abstractions (EmbPattern, EmbThread, ColorGroup)
- ✅ Format registry and dynamic discovery
- ✅ CLI tool with convert/info/validate/batch commands
- ✅ WASM support for browser usage
- ✅ Batch processing with optional parallelization
- ✅ Comprehensive test suite (522 tests)
- ✅ Documentation and examples

---

### Version 0.2.0 (Q2 2024) 📋

**Theme**: Enhanced Rendering & Visualization

**Format Improvements:**

- [ ] Add JAN format (Janome - bidirectional)
- [ ] Add PHC format (Pfaff - bidirectional)
- [ ] Add PHB format (Brother - bidirectional)
- [ ] Improve PES format version detection
- [ ] Add DSZ format support (ZSK)

**Rendering Features:**

- [ ] Realistic stitch rendering in PNG exports
- [ ] 3D stitch visualization (experimental)
- [ ] Texture-based stitch rendering
- [ ] Shadow and lighting effects for stitches
- [ ] Export animations (stitch-by-stitch playback)

**API Enhancements:**

- [ ] Pattern validation API (check for errors before writing)
- [ ] Stitch density analysis
- [ ] Pattern comparison utilities
- [ ] Metadata normalization across formats

**Performance:**

- [ ] Optimize DST encoding/decoding (SIMD)
- [ ] Reduce memory allocations in pattern building
- [ ] Benchmark suite expansion
- [ ] Lazy loading for large patterns

**Developer Experience:**

- [ ] Pattern builder with fluent API
- [ ] Format conversion presets (optimize for machine type)
- [ ] Better error messages with suggestions

---

### Version 0.3.0 (Q3 2024) 📋

**Theme**: Advanced Pattern Processing

**Pattern Analysis:**

- [ ] Stitch density maps
- [ ] Color usage statistics
- [ ] Pattern complexity scoring
- [ ] Thread consumption estimation
- [ ] Machine time estimation

**Pattern Transformations:**

- [ ] Automatic underlay generation
- [ ] Pull compensation
- [ ] Density adjustment
- [ ] Stitch type conversion (satin ↔ fill)
- [ ] Pattern merging and splitting

**Format Features:**

- [ ] Add EMD format (Elna/Melco Designer)
- [ ] Add PCD format (Pfaff Creative)
- [ ] Add VF3 format (Pfaff)
- [ ] Enhanced metadata preservation
- [ ] Format-specific optimization hints

**Quality Improvements:**

- [ ] Fuzz testing for all formats
- [ ] Property-based testing expansion
- [ ] CI/CD pipeline with multiple Rust versions
- [ ] Cross-platform testing (Windows, macOS, Linux)

---

### Version 0.4.0 (Q4 2024) 📋

**Theme**: Ecosystem & Community

**Ecosystem Integration:**

- [ ] Python bindings (PyO3)
- [ ] Node.js bindings (napi-rs)
- [ ] C/C++ FFI interface
- [ ] REST API server (optional binary)
- [ ] gRPC service (optional binary)

**Community Features:**

- [ ] Pattern sharing protocol
- [ ] Cloud storage integration examples
- [ ] Pattern marketplace integration guide
- [ ] Collaborative pattern editing (experimental)

**WASM Enhancements:**

- [ ] Interactive pattern editor (WASM + Canvas)
- [ ] Real-time format conversion in browser
- [ ] Pattern preview with zoom/pan
- [ ] Drag-and-drop file handling
- [ ] Mobile-responsive UI

**Documentation:**

- [ ] Video tutorials
- [ ] Interactive playground (docs.rs integration)
- [ ] Format specification reference
- [ ] Migration guides for popular libraries

---

### Version 1.0.0 (Q1 2025) 📋

**Theme**: Production Stability & Performance

**Stability:**

- [ ] API freeze (semver guarantees)
- [ ] Comprehensive security audit
- [ ] Performance audit
- [ ] Memory leak detection (Valgrind, MIRI)
- [ ] Edge case coverage (100% of known issues)

**Performance:**

- [ ] Zero-copy parsing where possible
- [ ] Streaming support for large files
- [ ] Memory-mapped file I/O (optional)
- [ ] SIMD optimizations (platform-specific)
- [ ] GPU acceleration (experimental)

**Enterprise Features:**

- [ ] Commercial support options
- [ ] SLA guarantees for critical bugs
- [ ] Enterprise licensing (dual-license model)
- [ ] Compliance certifications (if applicable)

**Quality Assurance:**

- [ ] 95%+ code coverage
- [ ] Continuous fuzzing (OSS-Fuzz)
- [ ] Performance regression testing
- [ ] Cross-platform compatibility matrix

---

## Long-Term Vision (2025+)

### Advanced Features

- **AI Integration**:
  - Automatic digitizing (bitmap → stitches)
  - Stitch quality prediction
  - Color palette suggestions
  - Pattern style transfer

- **Format Innovation**:
  - Define new open embroidery format
  - Support for embroidery machines with advanced features
  - Extended metadata (licensing, attribution)

- **Collaboration**:
  - Real-time collaborative editing
  - Version control integration (Git-like for patterns)
  - Pattern diff/merge tools

### Ecosystem Growth

- **Tooling**:
  - VS Code extension (pattern preview)
  - CLI with TUI (interactive mode)
  - Desktop GUI application

- **Integrations**:
  - CAD software plugins
  - Graphic design tool integrations
  - E-commerce platform connectors

- **Community**:
  - Pattern template library
  - Best practices guide
  - Format migration tools
  - Educational resources

---

## Feature Requests & Prioritization

### How Features Get Added

1. **Community Input**: GitHub Discussions, Issues, PRs
2. **Prioritization**: Based on impact, effort, and alignment with vision
3. **Specification**: Document requirements and design
4. **Implementation**: Develop with tests and documentation
5. **Review**: Code review, testing, benchmarking
6. **Release**: Include in next appropriate version

### Priority Criteria

**High Priority:**

- Critical bugs affecting data integrity
- Security vulnerabilities
- Format compatibility issues
- Performance regressions
- Requested by multiple users

**Medium Priority:**

- New format support (with bidirectional implementation)
- API improvements (non-breaking)
- Documentation enhancements
- Developer experience improvements

**Low Priority:**

- Nice-to-have features
- Experimental features
- Non-critical optimizations
- Cosmetic improvements

### Request a Feature

1. **Search**: Check existing issues and discussions
2. **Discuss**: Open a GitHub Discussion to gauge interest
3. **Propose**: Create a detailed issue with use case
4. **Contribute**: Submit a PR (see [CONTRIBUTING.md](CONTRIBUTING.md))

---

## Version Support Policy

### Supported Versions

- **Current Stable**: Active development, all updates
- **Previous Minor**: Critical security fixes only (6 months)
- **Older Versions**: Community support only

### Upgrade Path

- **Minor Versions**: Non-breaking, drop-in replacement
- **Major Versions**: Migration guide provided, deprecation warnings in prior version

### Breaking Changes

- **Avoid**: Minimize breaking changes within major versions
- **Announce**: Deprecation warnings in at least one minor version before removal
- **Document**: Clear migration guide in CHANGELOG.md
- **Support**: Provide tooling/scripts to assist migration where possible

---

## Dependencies & Maintenance

### Dependency Policy

- **Minimize**: Only add dependencies that provide significant value
- **Audit**: Regular security audits with `cargo audit`
- **Update**: Keep dependencies current (automated via Dependabot)
- **Evaluate**: Review all new dependencies for maintenance status

### Rust Version Policy

- **MSRV** (Minimum Supported Rust Version): Rust 1.70+
- **Update**: MSRV may increase in minor versions with 2 versions notice
- **Testing**: CI tests against stable, beta, and nightly

---

## Metrics & Success Criteria

### Version 0.2.0 Success Metrics

- [ ] 5+ new formats with full read/write support
- [ ] Realistic rendering quality matches commercial tools
- [ ] 10% performance improvement over 0.1.0
- [ ] 100+ GitHub stars
- [ ] 10+ community contributions

### Version 1.0.0 Success Metrics

- [ ] 25+ bidirectional formats
- [ ] 1000+ crates.io downloads per month
- [ ] 500+ GitHub stars
- [ ] 95%+ code coverage
- [ ] Zero known critical bugs
- [ ] 3+ production deployments

### Long-Term Success Metrics

- [ ] Industry-standard library for embroidery in Rust
- [ ] Active community with regular contributions
- [ ] Commercial adoption in embroidery software
- [ ] Integration in popular design tools

---

## Contributing to the Roadmap

We welcome input on the roadmap! Here's how to contribute:

1. **Feature Requests**: Open a GitHub Discussion
2. **Priority Feedback**: Comment on existing roadmap items
3. **Implementation**: Submit PRs for roadmap features
4. **Sponsorship**: Sponsor development of specific features

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

---

## Disclaimer

This roadmap is aspirational and subject to change based on:

- Community feedback
- Resource availability
- Technical feasibility
- Market demands
- Emerging standards

Dates are approximate. We prioritize quality over deadlines.

**Last Updated**: January 2024  
**Next Review**: Quarterly
