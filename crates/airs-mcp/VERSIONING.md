# Versioning Policy

This document outlines the versioning and release policies for airs-mcp starting from v0.2.0.

## Semantic Versioning

airs-mcp follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) with the format `MAJOR.MINOR.PATCH`.

### Version Format: `MAJOR.MINOR.PATCH`

#### MAJOR Version (`X.0.0`)
**When we increment the major version:**
- Breaking API changes that require code modifications
- Architectural redesigns affecting public interfaces
- Removal of deprecated features
- Changes requiring migration guides

**Examples of major changes:**
- Transport architecture refactoring (v0.1.x → v0.2.0)
- Module reorganization affecting import paths
- API signature changes in core interfaces
- Authentication system overhauls

**Requirements for major releases:**
- Comprehensive migration documentation
- Minimum 3-month advance notice (after v1.0.0)
- Clear deprecation timeline for removed features
- Updated examples demonstrating new patterns

#### MINOR Version (`0.X.0`)
**When we increment the minor version:**
- New features that are backward compatible
- New transport implementations (WebSocket, TCP, etc.)
- Enhanced authentication methods
- Performance improvements >20%
- New integration examples

**Examples of minor changes:**
- Adding WebSocket transport support
- New OAuth2 authentication flows
- Additional message handler implementations
- Enhanced error handling capabilities

**Requirements for minor releases:**
- Backward compatibility maintained
- New features documented with examples
- Performance improvements benchmarked
- No breaking changes to existing APIs

#### PATCH Version (`0.0.X`)
**When we increment the patch version:**
- Bug fixes that don't change APIs
- Documentation improvements
- Performance optimizations <20%
- Security patches
- Internal refactoring without API changes

**Examples of patch changes:**
- Fixing JSON-RPC serialization edge cases
- Documentation corrections and improvements
- Memory leak fixes
- Error message improvements

**Requirements for patch releases:**
- No breaking changes
- Backward compatibility guaranteed
- Focus on stability and reliability
- Rapid release for security issues

## Deprecation Policy

Starting from v0.2.0, we implement a **two-version deprecation cycle**:

### Deprecation Process

#### Version N: Initial Deprecation
- Feature marked as deprecated with `#[deprecated]` attribute
- Documentation updated with deprecation notices
- Clear guidance provided for replacement patterns
- Feature continues to work normally
- Warning messages in compilation/documentation

#### Version N+1: Strong Deprecation
- Stronger deprecation warnings
- Documentation prominently features replacement patterns
- Feature still works but with clear migration pressure
- Examples updated to use new patterns

#### Version N+2: Removal
- Deprecated feature completely removed
- Breaking change documented in changelog
- Migration guide updated with removal notice

### Deprecation Requirements

**For any deprecation:**
- Clear reason for deprecation documented
- Replacement pattern or alternative provided
- Migration path clearly explained
- Minimum timeline communicated to users
- Impact assessment on existing codebases

**Minimum deprecation timeline:**
- **Pre-1.0**: 2 versions (flexible for rapid development)
- **Post-1.0**: 6 months minimum between deprecation and removal

## Release Cadence

### Target Release Schedule
- **Major releases**: As needed for architectural improvements
- **Minor releases**: Monthly for new features
- **Patch releases**: As needed for bugs/security (within 1-2 weeks)

### Release Process

#### Pre-Release Requirements
1. **All tests passing** across supported Rust versions
2. **Documentation updated** including API docs and examples  
3. **Changelog updated** with user-facing changes
4. **Examples validated** and working correctly
5. **Performance benchmarks** run and compared
6. **Breaking changes documented** with migration guidance

#### Release Checklist
- [ ] Update version numbers in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Validate all integration examples
- [ ] Run full test suite and benchmarks
- [ ] Update documentation for new version
- [ ] Create release notes for GitHub
- [ ] Tag release and publish to crates.io

## API Stability Commitment

### Stable APIs (v0.2.0+)
APIs marked as stable will follow the deprecation policy:
- `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcNotification`
- `TransportClient` trait
- `MessageHandler<T>` trait
- Core protocol types in `protocol::types`

### Experimental APIs
APIs marked as experimental may change without deprecation:
- Features marked with `#[doc(hidden)]`
- Internal implementation details
- Undocumented APIs
- Features explicitly marked as experimental in documentation

### Compatibility Promise
Starting from v0.2.0:
- **Patch versions**: 100% backward compatible
- **Minor versions**: 100% backward compatible with new features
- **Major versions**: May include breaking changes with migration guides

## Version Support Policy

### Active Support
- **Current major version**: Full support with features, bugs, and security
- **Previous major version**: Security patches only for 6 months post-release

### End of Life
- Versions older than 2 major releases receive no support
- Security vulnerabilities in EOL versions will not be patched
- Users strongly encouraged to upgrade to supported versions

### Long Term Support (Future)
- After v1.0.0, we may designate specific versions as LTS
- LTS versions would receive security patches for 18 months
- LTS designation would be clearly communicated at release time

## Performance Policy

### Performance Regression
- **Major versions**: Performance regressions acceptable if documented
- **Minor versions**: No performance regressions >10% without justification
- **Patch versions**: No performance regressions >5%

### Performance Improvements
- Performance improvements >20% warrant minor version bump
- Benchmarking required for all performance-related changes
- Performance characteristics documented for each release

## Security Policy

### Security Updates
- Security patches released as patch versions immediately
- Security advisories published through GitHub Security Advisories
- Affected versions clearly documented
- Upgrade recommendations provided

### Vulnerability Disclosure
- Responsible disclosure process documented
- Security issues handled privately until patches available
- Credit given to security researchers per their preferences

## Communication

### Release Announcements
- GitHub releases with detailed changelog
- Documentation updates synchronized with releases
- Breaking changes communicated in advance when possible

### Community Feedback
- GitHub Issues for bug reports and feature requests
- GitHub Discussions for design conversations
- Responsive communication on version-related questions

---

## Effective Date

This versioning policy takes effect with the release of v0.2.0 on September 22, 2025.

Previous versions (v0.1.x) are governed by pre-policy practices and are now considered end-of-life.

For questions about this policy, please open a GitHub Discussion or Issue.