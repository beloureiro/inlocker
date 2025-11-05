#!/usr/bin/env node

import { readFileSync, writeFileSync, existsSync, unlinkSync } from 'fs';
import { execSync } from 'child_process';
import { join } from 'path';

// ============================================================================
// CONFIGURATION
// ============================================================================

const PROJECT_ROOT = process.cwd();
const CHANGELOG_PATH = join(PROJECT_ROOT, 'CHANGELOG.md');
const PACKAGE_JSON_PATH = join(PROJECT_ROOT, 'package.json');
const FLAG_FILE = join(PROJECT_ROOT, '.changelog-lock');

// ============================================================================
// MAIN FUNCTION
// ============================================================================

function main() {
  // PREVENT INFINITE LOOP: Check if flag file exists (already amended)
  if (existsSync(FLAG_FILE)) {
    unlinkSync(FLAG_FILE);
    console.log('✅ Changelog already included in commit (preventing loop)');
    return;
  }

  console.log('🔄 Updating changelog and version...');

  // 1. Get last commit message
  const commitMessage = getLastCommitMessage();
  console.log(`📝 Commit: ${commitMessage}`);

  // 2. Parse commit type
  const { type, scope, description, body, isBreaking } = parseCommit(commitMessage);
  console.log(`🏷️  Type: ${type}${isBreaking ? ' (BREAKING)' : ''}`);

  // 3. Determine if version should be bumped
  if (!shouldBumpVersion(type)) {
    console.log('⏭️  No version bump needed for this commit type.');
    return;
  }

  // 4. Get current version
  const currentVersion = getCurrentVersion();
  console.log(`📦 Current version: ${currentVersion}`);

  // 5. Calculate new version
  const newVersion = calculateNewVersion(currentVersion, type, isBreaking);
  console.log(`✨ New version: ${newVersion}`);

  // 6. Update CHANGELOG.md
  updateChangelog(newVersion, type, scope, description, body);
  console.log('📄 CHANGELOG.md updated');

  // 7. Update package.json
  updatePackageJson(newVersion);
  console.log('📦 package.json updated');

  // 8. Create flag file to prevent loop on amend
  writeFileSync(FLAG_FILE, new Date().toISOString());

  // 9. Add files and amend commit
  execSync('git add CHANGELOG.md package.json', { stdio: 'ignore' });
  execSync('git commit --amend --no-edit --no-verify', { stdio: 'ignore' });
  console.log('✅ Changelog and version included in current commit');
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Get the last commit message
 */
function getLastCommitMessage() {
  try {
    return execSync('git log -1 --pretty=%B', { encoding: 'utf-8' }).trim();
  } catch (error) {
    console.error('❌ Error reading commit message:', error.message);
    process.exit(1);
  }
}

/**
 * Parse commit message
 * Format: type(scope): description
 * Example: feat(auth): add login page
 * Breaking: feat!: or fix!: or BREAKING CHANGE: in body
 */
function parseCommit(message) {
  // Check for breaking change
  const isBreaking = message.includes('!:') || message.includes('BREAKING CHANGE');

  // Split message into first line and body
  const lines = message.split('\n');
  const firstLine = lines[0];
  const body = lines.slice(1).join('\n').trim();

  // Extract type, scope, and description from first line
  const conventionalCommitRegex = /^(\w+)(\(([^)]+)\))?(!)?:\s*(.+)/;
  const match = firstLine.match(conventionalCommitRegex);

  if (!match) {
    return {
      type: 'unknown',
      scope: null,
      description: message,
      body: '',
      isBreaking: false,
    };
  }

  return {
    type: match[1],           // feat or fix
    scope: match[3] || null,  // optional scope
    description: match[5],    // commit title (first line)
    body: body,               // commit body (remaining lines)
    isBreaking: isBreaking,
  };
}

/**
 * Determine if commit type should trigger version bump
 * ONLY feat and fix are recognized
 */
function shouldBumpVersion(type) {
  return type === 'feat' || type === 'fix';
}

/**
 * Get current version from package.json
 */
function getCurrentVersion() {
  try {
    const packageJson = JSON.parse(readFileSync(PACKAGE_JSON_PATH, 'utf-8'));
    return packageJson.version || '0.1.0';
  } catch (error) {
    console.error('❌ Error reading package.json:', error.message);
    return '0.1.0';
  }
}

/**
 * Calculate new version (MAJOR.MINOR.PATCH format)
 * Breaking changes: increments MAJOR (0.1.0 → 1.0.0)
 * feat: increments MINOR (0.1.0 → 0.2.0)
 * fix: increments PATCH (0.1.0 → 0.1.1)
 */
function calculateNewVersion(currentVersion, type, isBreaking) {
  const [major, minor, patch] = currentVersion.split('.').map(Number);

  if (isBreaking) {
    // MAJOR: Breaking change (0.1.0 → 1.0.0)
    return `${major + 1}.0.0`;
  } else if (type === 'feat') {
    // MINOR: New feature (0.1.0 → 0.2.0)
    return `${major}.${minor + 1}.0`;
  } else {
    // PATCH: Bug fix (0.1.0 → 0.1.1)
    return `${major}.${minor}.${patch + 1}`;
  }
}

/**
 * Update CHANGELOG.md with new entry
 */
function updateChangelog(version, type, scope, description, body) {
  const date = new Date().toISOString().split('T')[0]; // YYYY-MM-DD
  const changeType = getChangeTypeLabel(type);
  const scopeText = scope ? `**${scope}:** ` : '';

  // Format the body: convert to indented bullet points if it contains line breaks
  let formattedBody = '';
  if (body && body.trim()) {
    const bodyLines = body.split('\n').filter(line => line.trim());
    formattedBody = bodyLines.map(line => {
      // If line already starts with -, keep it as is
      if (line.trim().startsWith('-')) {
        return `  ${line.trim()}`;
      }
      // Otherwise, add a bullet point
      return `  - ${line.trim()}`;
    }).join('\n');
  }

  // Create new entry
  const newEntry = `
## [${version}] - ${date}

### ${changeType}

- ${scopeText}${description}
${formattedBody ? '\n' + formattedBody : ''}
`;

  // Read existing changelog or create new one
  let changelog = '';
  if (existsSync(CHANGELOG_PATH)) {
    changelog = readFileSync(CHANGELOG_PATH, 'utf-8');
  } else {
    // Create initial CHANGELOG.md if it doesn't exist
    changelog = `# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`;
  }

  // Insert new entry after the header
  const lines = changelog.split('\n');
  const headerEndIndex = lines.findIndex(line => line.startsWith('##'));

  if (headerEndIndex === -1) {
    // No existing entries, append to end
    changelog += newEntry;
  } else {
    // Insert before first existing entry
    lines.splice(headerEndIndex, 0, newEntry.trim() + '\n');
    changelog = lines.join('\n');
  }

  writeFileSync(CHANGELOG_PATH, changelog);
}

/**
 * Get human-readable change type label
 */
function getChangeTypeLabel(type) {
  const labels = {
    feat: 'Features',
    fix: 'Bug Fixes',
  };
  return labels[type] || 'Changes';
}

/**
 * Update version in package.json
 */
function updatePackageJson(newVersion) {
  const packageJson = JSON.parse(readFileSync(PACKAGE_JSON_PATH, 'utf-8'));
  packageJson.version = newVersion;
  writeFileSync(PACKAGE_JSON_PATH, JSON.stringify(packageJson, null, 2) + '\n');
}

// ============================================================================
// RUN
// ============================================================================

main();
