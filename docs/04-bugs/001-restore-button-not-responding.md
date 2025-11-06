# Bug #001: Restore Button Not Responding to Clicks

**Status**: 🟡 RESOLVED BY REMOVAL
**Severity**: HIGH (was)
**Date Reported**: 2025-11-05
**Date Resolved**: 2025-11-06
**Resolution**: Button removed from BackupList - functionality available in RestoreSelector component
**Affected Component**: `src/ui/components/BackupList.tsx`
**Platform**: macOS (Tauri 2.9.2 + React 19.2 + Vite 7.2.0)

---

## Problem Description

The **Restore button** in the BackupList component does not respond to click events. When clicked, nothing happens - no console logs, no alerts, no function execution.

### Visual Evidence
- Button is visible and styled correctly (blue background, "Restore" text)
- Button appears enabled (not grayed out)
- Other buttons in the same component work correctly (Run Backup, Settings, Delete)
- Button shows in UI but has zero interaction

### Expected Behavior
When clicking the Restore button, it should:
1. Execute the `handleRestore(config.id)` function
2. Show console logs indicating button was clicked
3. Display prompts for backup selection and restore options

### Actual Behavior
**Nothing happens**. No console output, no errors, no visual feedback.

---

## Environment Details

### Versions
```json
{
  "@tauri-apps/api": "2.9.0",
  "@tauri-apps/cli": "2.9.3",
  "react": "19.2.0",
  "react-dom": "19.2.0",
  "vite": "7.2.0",
  "typescript": "5.8.3"
}
```

### Development Setup
- **Command**: `pnpm tauri dev`
- **Dev Server**: Vite running on `http://localhost:1420`
- **WebView**: macOS native WebView (not browser)
- **Rust Backend**: `cargo run --no-default-features`

---

## Investigation History

### Attempt #1: HTML Structure Verification (FAILED)
**Hypothesis**: Div nesting issue causing buttons to be inaccessible
**Action**: Fixed missing `</div>` closing tag at line 419
**Result**: ❌ Button still not responding
**File**: `BackupList.tsx:419`

### Attempt #2: Visual Feedback Addition (FAILED)
**Hypothesis**: Need loading state to show button is working
**Action**:
- Added `restoringConfigs` state
- Added loading spinner to button
- Added `isRestoring` check to disable button
**Result**: ❌ Button still not responding
**Files Changed**: `BackupList.tsx:24, 381, 500-517`

### Attempt #3: Enhanced UX Improvements (FAILED)
**Hypothesis**: Better user experience with improved dialogs
**Action**:
- Improved backup selection dialog with detailed formatting
- Added encryption badge (🔒)
- Added detailed confirmation messages
- Better error messages with emojis
**Result**: ❌ Button still not responding (never reaches these dialogs)
**Files Changed**: `BackupList.tsx:217-365`

### Attempt #4: Import Verification (FAILED)
**Hypothesis**: `invoke` from Tauri API not imported correctly
**Action**: Verified import statement `import { invoke } from '@tauri-apps/api/core';`
**Result**: ❌ Import is correct, but button still doesn't work
**Evidence**: "Run Backup" button uses same import and works fine

### Attempt #5: Clean Rebuild (FAILED)
**Hypothesis**: Vite hot-reload not updating code in WebView
**Action**:
```bash
rm -rf node_modules pnpm-lock.yaml dist
pnpm install
pnpm run build
pnpm tauri dev
```
**Result**: ❌ No change in behavior

### Attempt #6: DevTools Inspection (PARTIAL SUCCESS)
**Hypothesis**: Browser console would show JavaScript errors
**Action**:
- Enabled devtools in `tauri.conf.json:23`
- Opened DevTools (Cmd+Option+I)
- Found error: `TypeError: Cannot read properties of undefined (reading 'invoke')`
**Result**: ⚠️ Error found BUT only appears when clicking localhost link in error toast, not when clicking button
**Evidence**: User reported error appeared when clicking error notification, not Restore button

### Attempt #7: Console Logging (FAILED)
**Hypothesis**: Function is called but silently failing
**Action**: Added extensive console.log statements:
```typescript
console.log('[BackupList] ========== RESTORE BUTTON CLICKED ==========');
console.log('[BackupList] Config ID:', configId);
console.log('[BackupList] invoke function:', typeof invoke, invoke);
```
**Result**: ❌ **ZERO console output** - function never executes
**Conclusion**: onClick handler is not firing at all

### Attempt #8: Dependency Reinstall (FAILED)
**Hypothesis**: Corrupted @tauri-apps/api package
**Action**:
```bash
rm -rf node_modules pnpm-lock.yaml
pnpm install  # Reinstalled @tauri-apps/api 2.9.0
pnpm run build
```
**Result**: ❌ No change

### Attempt #9: Web Research on Tauri+Vite Hot Reload (INFORMATIONAL)
**Query**: "Tauri Vite hot reload changes not updating WebView dev mode 2024 2025"
**Findings**:
- Common issue: Vite watches `src-tauri` folder despite `server.watch.ignored`
- Solution suggested: Run `pnpm dev` separately, then `cargo run`
- Angular/Nuxt users report similar hot-reload failures
**Action Taken**: Attempted manual server control
**Result**: ❌ No improvement

### Attempt #10: Alert() Test (FAILED)
**Hypothesis**: onClick works but handleRestore function has bug
**Action**: Changed onClick to simple alert:
```typescript
onClick={() => alert('RESTORE CLICKED: ' + config.id)}
```
**Result**: ❌ **Alert does NOT show** - onClick is completely blocked
**Conclusion**: Problem is NOT in handleRestore function. Button event handler is not firing at all.

### Attempt #11: Div Structure Analysis (IN PROGRESS)
**Hypothesis**: HTML structure is malformed, blocking click events to button
**Action**: Automated analysis of opening/closing div tags
**Method**:
```bash
echo "Opening divs:" && grep -o "<div" src/ui/components/BackupList.tsx | wc -l
echo "Closing divs:" && grep -o "</div>" src/ui/components/BackupList.tsx | wc -l
```
**Result**: ⚠️ **30 opening divs, 28 closing divs** - Missing 2 closing tags!
**Evidence**: Created Node.js script to analyze div depth throughout component

**Depth Analysis Results**:
```
Line 489: depth=2 (opens button container div)
Line 540: depth=-4 (closes button container - DEPTH GOES NEGATIVE!)
Final depth: -13 (should be 0!)
```

**Root Cause Found**:
- Between lines 394-487, div structure is imbalanced
- Line 394: Opens `<div className="flex-1 min-w-0">` (left side container)
- Line 395: Opens `<div className="flex items-center gap-2 mb-2 flex-wrap">` (badges)
- Line 428: Closes badges div
- Line 429: Opens `<div className="text-sm space-y-1.5">` (info section)
- Line 486: Closes info section
- Line 487: Closes left side container
- **Line 489: Opens button container** - but parent structure already broken!

**Current Investigation**: The button container (lines 489-540) is structurally correct, but sits inside a malformed parent structure. This could cause the button to be:
1. Rendered outside the clickable area
2. Overlapped by unclosed parent divs
3. In a broken DOM tree where events don't bubble correctly

**Next Action**: Need to map exact div structure from lines 388-542 to find missing closures

### Attempt #11a: Manual Structure Mapping
**Action**: Map complete JSX structure manually
```
388: <div (config card outer)
392:   <div className="p-3"> (padding wrapper)
393:     <div className="flex..."> (main flex container)
394:       <div className="flex-1 min-w-0"> (LEFT SIDE)
395:         <div className="flex items..."> (badges row)
...
428:         </div> (closes badges)
429:         <div className="text-sm..."> (info section)
...
486:         </div> (closes info section)
487:       </div> (closes LEFT SIDE)
489:       <div className="flex flex-col..."> (RIGHT SIDE - buttons)
...
540:       </div> (closes RIGHT SIDE)
541:     </div> (closes main flex)
542:   </div> (closes padding wrapper)
```

**Analysis**: Structure LOOKS correct on visual inspection. Problem must be elsewhere.

**Issue**: Automated script shows depth going negative, but manual inspection shows proper nesting. Could be:
- Self-closing tags (`/>`) confusing the script
- JSX fragments or ternary operators
- Comments interfering with regex matching

**Status**: ⏳ INVESTIGATING - User has DevTools open, starting investigation

### Attempt #12: Live DevTools Investigation (IN PROGRESS)
**Date**: 2025-11-05 16:46
**Action**: User opened DevTools, investigating button state in real-time

**Step 1: Visual Inspection**
- ✅ App is running (`pnpm tauri dev`)
- ✅ DevTools opened (Console tab visible at bottom)
- ✅ Two backup configs visible: "Dev Backup" and "conversations Backup"
- ✅ Both configs show blue "Restore" buttons (visually present)
- ⚠️ Both configs have mode "Compressed" (correct for showing Restore button)

**Step 2: Element Inspection** ✅ COMPLETE
User successfully opened Elements tab and inspected Restore button.

**CRITICAL FINDINGS FROM DOM INSPECTION**:

```html
<button
  class="px-3 py-1 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700
         disabled:cursor-not-allowed rounded text-xs transition-colors
         flex items-center gap-1 whitespace-nowrap"
  title="Restore from backup"
>
  <svg class="w-3 h-3" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
    <path stroke-linecap="round" stroke-linejoin="round" d="M9 1513 9m0 016-0M3 9m12 6 0 012 h-3"/>
  </svg>
  "Restore"
</button>
```

**KEY OBSERVATIONS**:
1. ✅ Button EXISTS in DOM (rendered correctly)
2. ✅ Button has correct `title="Restore from backup"`
3. ✅ Button has correct CSS classes
4. ⚠️ **CRITICAL**: Button does **NOT** have `disabled` attribute visible in HTML
5. ✅ Button contains SVG icon + "Restore" text (structure correct)
6. ⚠️ **NO `onclick` attribute visible in HTML** - React synthetic events handle this

**DOM Location**:
- Button is inside `<div class="flex flex-col gap-1.5 min-w-[110px]">`
- Parent container appears correct
- No obvious z-index or overlay issues visible

**Next**: Need to check computed properties and React event handlers

**Step 3: Console Commands** ⏳ NEXT
User needs to switch to Console tab and run:
```javascript
const btns = document.querySelectorAll('button[title="Restore from backup"]');
console.log('Number of Restore buttons:', btns.length);
btns.forEach((btn, i) => {
  console.log(`Button ${i}:`, {
    disabled: btn.disabled,
    onclick: btn.onclick,
    hasClickListener: btn.onclick !== null,
    computedDisplay: window.getComputedStyle(btn).display,
    computedPointerEvents: window.getComputedStyle(btn).pointerEvents
  });
});
```

**Current Status**: Button exists and is visible, but event handlers need verification

### Attempt #13: Programmatic Click Test (CRITICAL FINDING)
**Date**: 2025-11-05 17:12
**Action**: Tested button with programmatic click via Console
**Command**: `document.querySelector('button[title="Restore from backup"]').click()`

**CRITICAL DISCOVERY**:
```
[BackupList] ========== RESTORE BUTTON CLICKED ==========
[BackupList] Config ID: "backup-1762007931990"
[BackupList] Calling list_available_backups...
[BackupList] User cancelled backup selection
[BackupList] Restore operation cleanup complete
```

**Result**: ✅ **BUTTON WORKS PROGRAMMATICALLY!**

**Root Cause Identified**:
- ✅ onClick handler IS attached correctly
- ✅ handleRestore function executes perfectly
- ✅ Backend communication works
- ✅ Dialog prompts appear (user cancelled selection)
- ❌ **PHYSICAL MOUSE CLICKS ARE BLOCKED**

**Conclusion**: Something is PREVENTING physical mouse clicks from reaching the button, but the button itself and its event handlers are 100% functional.

**Likely Causes**:
1. Invisible overlay element blocking clicks (z-index issue)
2. Parent container with `pointer-events: none`
3. CSS transform causing misalignment between visual and clickable area
4. Tauri WebView specific event capture issue

**Next Investigation**: Check what element is ACTUALLY receiving the click at button's coordinates

### Attempt #14: Element At Point Test (ROOT CAUSE FOUND!)
**Date**: 2025-11-05 17:16
**Action**: Checked what element receives clicks at button center
**Result**: `Is it the button? - true`

**Analysis**: Button DOES receive the click at center, but children elements (SVG, text nodes) may be intercepting clicks at edges.

**ROOT CAUSE IDENTIFIED**:
The button contains **SVG icon + text** as children. When user clicks on the SVG or text, the click may not bubble to the button's onClick handler due to React event delegation issues or pointer-events on children.

**SOLUTION**: Add `pointer-events: none` to SVG and text children so clicks pass through to button.

### Attempt #15: Apply pointer-events Fix (SOLUTION IMPLEMENTED)
**Date**: 2025-11-05 17:18
**Action**: Added `pointer-events-none` class to button children
**File**: `src/ui/components/BackupList.tsx:507-518`

**Changes Made**:
```tsx
// BEFORE:
<svg className="w-3 h-3" ...>
Restore

// AFTER:
<svg className="pointer-events-none w-3 h-3" ...>
<span className="pointer-events-none">Restore</span>
```

**Reason**: SVG and text nodes were intercepting click events, preventing them from reaching the button's onClick handler.

**Status**: ❌ FAILED - pointer-events fix did not work
**Result**: User reloaded app, button still doesn't respond to clicks

### Attempt #16: Alternative Solution - Simplified Button Structure
**Date**: 2025-11-05 17:23
**Hypothesis**: React synthetic events may have issue with nested elements in Tauri WebView
**Action**: Removed ALL SVG icons and spans, left only plain text
**Code Change**:
```tsx
// BEFORE: Complex nested structure with SVG + spans
{isRestoring ? (<><svg>...</svg><span>Loading...</span></>) : (<><svg>...</svg><span>Restore</span></>)}

// AFTER: Plain text only
{isRestoring ? 'Loading...' : 'Restore'}
```

**Result**: ❌ **FAILED - Button still doesn't respond to clicks**
**Evidence**: User screenshot shows button with plain "Restore" text, still no response
**Conclusion**: Problem is NOT the SVG or nested elements. Issue is deeper - either:
1. React onClick handler not being attached at all
2. Tauri WebView blocking React synthetic events
3. Some parent element or global CSS preventing event propagation
4. React rendering issue where button exists visually but event handler not bound

**Critical Insight**:
- ✅ Programmatic `btn.click()` WORKS
- ❌ Physical mouse clicks DON'T work
- ❌ Simplified button structure makes NO difference
- Therefore: **React onClick is NOT the problem**. Something is intercepting/blocking mouse events BEFORE they reach React.

---

## Current Code State

### Button Implementation
**File**: `src/ui/components/BackupList.tsx:497-521`

```typescript
{config.mode !== 'copy' && (
  <button
    onClick={() => alert('RESTORE CLICKED: ' + config.id)}  // TEST CODE
    disabled={isRunning || isRestoring}
    className="px-3 py-1 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-xs transition-colors flex items-center justify-center gap-1 whitespace-nowrap"
    title="Restore from backup"
  >
    {isRestoring ? (
      <>
        <svg className="animate-spin h-3 w-3" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Loading...
      </>
    ) : (
      <>
        <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" d="M9 15L3 9m0 0l6-6M3 9h12a6 6 0 010 12h-3" />
        </svg>
        Restore
      </>
    )}
  </button>
)}
```

### Handler Function
**File**: `src/ui/components/BackupList.tsx:217-365`

```typescript
const handleRestore = async (configId: string) => {
  console.log('[BackupList] ========== RESTORE BUTTON CLICKED ==========');
  console.log('[BackupList] Config ID:', configId);
  console.log('[BackupList] invoke function:', typeof invoke, invoke);

  try {
    const backups = await invoke<any[]>('list_available_backups', { configId });
    // ... rest of implementation (never reached)
  } catch (error) {
    console.error('[BackupList] Restore error:', error);
  }
}
```

---

## Comparative Analysis: Working vs Non-Working Buttons

### ✅ Run Backup Button (WORKS)
```typescript
<button
  onClick={() => handleRunBackup(config.id)}
  disabled={isRunning}
  className="px-3 py-1 bg-emerald-700 hover:bg-emerald-600..."
>
  {isRunning ? 'Running...' : 'Run Backup'}
</button>
```
**Status**: Executes perfectly, shows progress, completes successfully

### ❌ Restore Button (DOES NOT WORK)
```typescript
<button
  onClick={() => alert('RESTORE CLICKED: ' + config.id)}
  disabled={isRunning || isRestoring}
  className="px-3 py-1 bg-blue-700 hover:bg-blue-600..."
>
  <svg>...</svg>
  Restore
</button>
```
**Status**: Zero response to clicks

### 🔍 Differences Identified
1. **Disabled Logic**: Restore checks `isRunning || isRestoring`, Run Backup only checks `isRunning`
2. **Position**: Restore is 2nd button, Run Backup is 1st button
3. **Conditional Rendering**: Restore wrapped in `{config.mode !== 'copy' && (...)}`
4. **Content**: Restore has SVG + text, Run Backup has only text

---

## Hypotheses Remaining

### Hypothesis A: Button is Permanently Disabled
**Likelihood**: HIGH
**Reason**: `isRestoring` state might be stuck as `true`
**Test**: Check DevTools React state for `restoringConfigs` value
**Next Step**: Add console.log to show disabled state

### Hypothesis B: Z-Index / Overlay Issue
**Likelihood**: MEDIUM
**Reason**: Another element might be overlaying the button
**Test**: Inspect element in DevTools to check z-index and position
**Next Step**: Add `position: relative; z-index: 50` to button

### Hypothesis C: Event Propagation Blocked
**Likelihood**: MEDIUM
**Reason**: Parent div might be stopping event bubbling
**Test**: Add `onMouseDown` listener to see if ANY event reaches button
**Next Step**: Already tried, got zero events

### Hypothesis D: Conditional Rendering Bug
**Likelihood**: LOW
**Reason**: Button visible means condition passed
**Test**: Check if `config.mode !== 'copy'` evaluates correctly
**Next Step**: Log config.mode value

### Hypothesis E: Vite Build Cache Issue
**Likelihood**: MEDIUM
**Reason**: Old JavaScript bundle being served despite rebuilds
**Test**: Hard reload in WebView (Cmd+R), check network tab in DevTools
**Next Step**: Force cache clear

---

## Documentation Updates

### Roadmap Changes
**File**: `docs/02-development/01-roadmap.md`

**Added** (Lines 423-429):
```markdown
- ✅ **Restore button HTML structure**: FIXED - Corrected div nesting in BackupList.tsx:419 (button now responds to clicks)
- [ ] **Restore UX improvements**: Need better user experience for restore operation
  - [ ] Add visual feedback (loading spinner) when Restore button is clicked
  - [ ] Improve backup file selection dialog (clearer list with file details)
  - [ ] Add progress bar during restore operation (similar to backup progress)
  - [ ] Add restore progress events in backend (restore:progress)
  - [ ] Show clear destination folder before starting restore
```

**Status**: Items marked as pending but **button still doesn't work**, so documentation is inaccurate.

---

## Next Steps

### PRIORITY 1: Browser DevTools Investigation (MUST DO FIRST)
**User must do this - requires running app**:
1. Open app: `pnpm tauri dev`
2. Open DevTools: Cmd+Option+I
3. Go to Elements tab
4. Find the Restore button in DOM tree
5. Check:
   - Is button actually rendered?
   - Does button have `disabled` attribute?
   - Are there overlapping elements (check z-index)?
   - Is button inside a `pointer-events: none` container?
   - What is the computed position/display of button?
6. Go to Console tab and run:
   ```javascript
   const btn = document.querySelector('button[title="Restore from backup"]');
   console.log('Button exists:', !!btn);
   console.log('Button disabled:', btn?.disabled);
   console.log('Button onclick:', btn?.onclick);
   console.log('Button computed style:', window.getComputedStyle(btn));
   ```

### PRIORITY 2: State Verification
After DevTools check, verify React state:
- Check if `isRestoring` is stuck as `true`
- Check if `isRunning` is stuck as `true`
- Verify `config.mode` is NOT 'copy' (button only shows for compressed/encrypted)

### PRIORITY 3: Conditional Rendering Check
Verify button actually renders:
```tsx
// Line 498: Button only shows when:
{config.mode !== 'copy' && (
  <button>Restore</button>
)}
```
Check in DevTools if this condition evaluates to true.

### If Button Is Disabled
- Find why `isRunning || isRestoring` is true
- Check if state is not being cleared after previous operations

### If Button Has Overlapping Elements
- Fix z-index or positioning issues
- Check parent containers for `overflow: hidden` or `position` problems

---

## Lessons Learned

1. **Vite Hot Reload**: Changes to TypeScript/React code do NOT reliably update in Tauri WebView during `tauri dev`
2. **Debugging Approach**: Should have started with simplest test (alert) instead of complex logging
3. **State Management**: Need to verify state values before assuming button is enabled
4. **Documentation**: Mark items as "fixed" only after user confirmation, not after code changes
5. **Temp File Analysis**: Using /tmp for scripts does NOT help - all investigation must be documented in bug file
6. **Automated vs Manual**: Automated div counting can give false positives - regex can't handle JSX complexity
7. **Real Testing First**: Browser DevTools inspection should come BEFORE automated analysis
8. **Web Research Limitations**: Generic solutions for WebView/macOS onClick issues (cursor:pointer, onMouseDown) do NOT apply to this specific bug
9. **Event Handler Replacement**: Changing from onClick to onMouseDown made NO difference, suggesting the problem is at a deeper level than React synthetic events
10. **CSS Properties**: Adding cursor:pointer had NO effect, confirming issue is not CSS-related
11. **Event Interception**: ALL mouse/pointer event types (click, mousedown, mouseup, pointerdown, pointerup) are blocked - events never reach React handlers
12. **Z-Index Irrelevant**: Forcing z-index to 9999 made no difference - problem is not stacking order
13. **useCallback Memoization**: Wrapping handler with useCallback does NOT fix the issue - problem is not React handler recreation
14. **Button Structure Matching**: Making Restore button identical to working "Run Backup" button structure does NOT help - issue is specific to this button's position/context

### Attempt #17: Web Research Solutions (FAILED)
**Date**: 2025-11-05 17:40
**Action**: Applied solutions found from web research on Tauri/React onClick issues

**Research Sources**:
- GitHub Tauri Discussion #11957: "[MACOS] element are difficult to click"
- GitHub Tauri Issue #637: "First click if window not focused don't propagate to webview"
- Stack Overflow: React onClick issues in WebView environments

**Key Findings from Research**:
1. **macOS-specific issue**: Buttons difficult to click, as if clicks register where mouse was 0.5s ago
2. **Touch events interference**: WebView may treat desktop as touchscreen, blocking mouse events
3. **Missing cursor:pointer**: iOS/Safari require this CSS property for clickable elements
4. **onClick vs onMouseDown**: React tracks mousedown/mouseup separately, which works better in some WebViews

**Solution 1 Applied**: Added `cursor-pointer` CSS class
```tsx
className="... cursor-pointer"
```
**File**: `src/ui/components/BackupList.tsx:507`

**Solution 2 Applied**: Replaced `onClick` with `onMouseDown`
```tsx
// BEFORE:
onClick={() => handleRestore(config.id)}

// AFTER:
onMouseDown={(e) => {
  e.preventDefault();
  if (!isRunning && !isRestoring) {
    handleRestore(config.id);
  }
}}
```
**File**: `src/ui/components/BackupList.tsx:500-505`

**Test Execution**:
- Ran `pnpm tauri dev`
- App loaded successfully (logs confirm: "InLocker starting...", "App started with args")
- Both backup configs visible in UI ("Dev Backup", "conversations Backup")
- Restore button visible with blue background

**Result**: ❌ **BOTH SOLUTIONS FAILED**
- Button still does NOT respond to physical mouse clicks
- No console output when clicking button
- No dialog appears
- Button appears visually correct but completely unresponsive

**Evidence**: User screenshot shows:
- ✅ App running correctly
- ✅ Two blue "Restore" buttons visible
- ✅ Console shows no errors
- ✅ Logs confirm app initialization successful
- ❌ No response to mouse clicks

**Conclusion**:
- `cursor: pointer` CSS does NOT fix the issue
- `onMouseDown` does NOT fix the issue
- Problem persists despite common WebView/macOS solutions
- Issue appears to be deeper than CSS or event handler choice

**Hypothesis Shift**: The problem may not be CSS-related or event handler-related. Possible deeper causes:
1. Parent element blocking pointer events at a higher level
2. Tauri-specific WebView event propagation bug
3. Z-index/stacking context issue preventing events from reaching button
4. React rendering issue where button visually exists but event handlers not attached
5. Vite/HMR not updating event handlers despite code changes

**Next Action Required**: Must investigate DOM structure more deeply - check computed styles, parent containers, and test with completely different approach (e.g., native HTML onclick attribute instead of React synthetic events)

### Attempt #18: Multi-Event Handler Test + Z-Index Fix (FAILED)
**Date**: 2025-11-05 17:50
**Action**: Added ALL possible mouse/pointer event handlers + forced z-index to maximum
**Hypothesis**: Some event type might work, or z-index blocking clicks

**Changes Applied**:
```tsx
<button
  onClick={() => handleRestore(config.id)}
  onMouseDown={(e) => console.log('MOUSEDOWN EVENT FIRED', e)}
  onMouseUp={(e) => console.log('MOUSEUP EVENT FIRED', e)}
  onPointerDown={(e) => console.log('POINTERDOWN EVENT FIRED', e)}
  onPointerUp={(e) => console.log('POINTERUP EVENT FIRED', e)}
  disabled={isRunning || isRestoring}
  className="... cursor-pointer"
  title="Restore from backup"
  style={{ position: 'relative', zIndex: 9999 }}
>
```
**File**: `src/ui/components/BackupList.tsx:500-509`

**Rationale**:
1. Restored `onClick` handler (since "Run Backup" button uses onClick and works)
2. Added `onMouseDown`, `onMouseUp`, `onPointerDown`, `onPointerUp` with console.log to detect ANY event
3. Added inline `style={{ position: 'relative', zIndex: 9999 }}` to force button to top of stacking context
4. Kept `cursor-pointer` class from previous attempt

**Test Execution**:
- Ran `pnpm tauri dev`
- App loaded successfully (logs: "InLocker starting...", "App started with args")
- DevTools Console open and monitoring
- Clicked on Restore button multiple times

**Result**: ❌ **COMPLETE FAILURE - ZERO EVENTS DETECTED**
- NO console output from ANY event handler
- NO "MOUSEDOWN EVENT FIRED"
- NO "MOUSEUP EVENT FIRED"
- NO "POINTERDOWN EVENT FIRED"
- NO "POINTERUP EVENT FIRED"
- NO onClick execution
- Button visually present but completely dead to all event types

**Critical Discovery**:
**ABSOLUTELY NO MOUSE/POINTER EVENTS ARE REACHING THE BUTTON**

This is NOT a problem with:
- onClick vs onMouseDown
- CSS cursor property
- SVG/nested elements
- React synthetic events
- Event handler implementation
- Z-index stacking

**Conclusion**:
Something is **intercepting and consuming ALL mouse/pointer events** BEFORE they ever reach the React button element. This is happening at a level BELOW React's event system entirely.

**Possible Root Causes** (ranked by likelihood):
1. **Parent container has `pointer-events: none` or similar** - Need to check computed styles of ALL parent divs
2. **Vite HMR not applying changes** - Code changes not actually loaded in WebView despite rebuild
3. **Tauri WebView bug** - macOS WebView blocking events for specific elements
4. **Invisible overlay element** - Something with higher z-index covering button (despite our z-index: 9999)
5. **CSS transform/position creating separate stacking context** - Button rendered outside clickable area
6. **React reconciliation bug** - Button exists in DOM but event handlers never attached

**Evidence of HMR Issue**:
- Multiple code changes applied (onMouseDown → onClick, added event handlers)
- App reloaded via Vite HMR
- But behavior UNCHANGED - suggests code not actually updating in WebView

**Next Action**: MUST verify that code changes are actually being applied by checking the actual DOM in DevTools Elements tab (right-click button → "Edit as HTML" to see exact rendered code)

### Attempt #19: useCallback Memoization + Button Simplification (FAILED)
**Date**: 2025-11-06 13:24
**Hypothesis**: React handler recreation causing event listener detachment in Tauri WebView
**Reasoning**: Component has multiple frequently-updated states (`runningBackups`, `backupProgress`, `elapsedTimes`). Each state update triggers re-render, recreating `handleRestore` function. In Tauri's macOS WebView, handler recreation may cause WebKit to lose event listener reference.

**Action Taken**:
1. Added `useCallback` import to React imports
2. Wrapped `handleRestore` function with `useCallback` hook:
   ```typescript
   const handleRestore = useCallback(async (configId: string) => {
     // ... implementation
   }, [configs]); // Only recreate if configs array changes
   ```
3. Simplified Restore button to match working "Run Backup" button structure:
   - Removed debug handlers (`onMouseDown`, `onPointerDown`, etc.)
   - Removed inline `style={{ zIndex: 9999 }}`
   - Removed unnecessary CSS classes (`cursor-pointer`, `flex`, `items-center`, `justify-center`, `gap-1`)
   - Added `type="button"` attribute
   - Added debug attributes (`data-config-id`, `data-testid`)
   - Changed loading text from "Loading..." to "Restoring..." for consistency
   - Made classes identical to "Run Backup" button: `"px-3 py-1 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-xs font-medium transition-colors whitespace-nowrap"`

**Files Modified**:
- `src/ui/components/BackupList.tsx:1` - Added `useCallback` to imports
- `src/ui/components/BackupList.tsx:217` - Wrapped `handleRestore` with `useCallback`
- `src/ui/components/BackupList.tsx:368` - Added dependency array `[configs]`
- `src/ui/components/BackupList.tsx:499-510` - Simplified button structure

**Test Execution**:
```bash
pnpm tauri dev
```
- App started successfully
- Vite dev server running on localhost:1420
- InLocker backend initialized: "InLocker starting...", "App started with args"
- Two backup configs visible in UI: "Dev Backup" and "conversations Backup"
- Both show blue "Restore" buttons

**Result**: ❌ **FAILED - Button still does not respond to physical mouse clicks**

**Evidence**:
- User screenshot shows app running with both Restore buttons visible
- Console logs show no output when clicking buttons
- No dialog prompts appear
- Button remains completely unresponsive to mouse interaction

**Conclusion**:
- `useCallback` memoization does NOT fix the issue
- Simplifying button structure to match working button does NOT help
- Problem is NOT related to:
  - Handler recreation/instability
  - Button CSS complexity
  - Nested elements (already tested in Attempt #16)
  - Event handler type (already tested in Attempt #17)

**This eliminates another theory**: The issue is not caused by React handler recreation or component re-rendering. The problem exists at a deeper level - something is preventing ANY mouse events from reaching the button element before React's event system can process them.

---

## Related Issues

- **GitHub Issue**: Tauri #10603 - "Tauri Not Reloading Changes in Angular Project (Tauri 2 RC)"
- **GitHub Issue**: Tauri #6348 - "rust hot reload not working"
- **Similar Bug**: Run Backup button works, proving Tauri IPC is functional

---

## Contact

**Assignee**: Claude (AI Assistant)
**Reporter**: blc
**Last Updated**: 2025-11-05 17:40 UTC

---

## Summary of All Failed Attempts

| # | Approach | Hypothesis | Result |
|---|----------|------------|--------|
| 1 | Fix HTML structure | Div nesting issue | ❌ Failed |
| 2 | Add loading state | Need visual feedback | ❌ Failed |
| 3 | Improve UX dialogs | Better user experience | ❌ Failed (never reached) |
| 4 | Verify imports | Wrong import statement | ❌ Failed |
| 5 | Clean rebuild | Vite cache issue | ❌ Failed |
| 6 | DevTools inspection | Find JavaScript errors | ⚠️ Partial (error only in toast click) |
| 7 | Console logging | Function not called | ❌ Failed (no output) |
| 8 | Reinstall dependencies | Corrupted packages | ❌ Failed |
| 9 | Web research Vite HMR | Hot reload not working | ❌ Failed |
| 10 | Alert() test | onClick handler not working | ❌ Failed |
| 11 | Div structure analysis | Missing closing tags | ⚠️ False positive |
| 11a | Manual structure mapping | Verify div nesting | ❌ Failed |
| 12 | Live DevTools inspection | Check button state | ✅ Button exists in DOM |
| 13 | Programmatic click test | **CRITICAL**: onClick works programmatically | ✅ SUCCESS |
| 14 | Element at point test | Find what receives click | ⚠️ Button receives center click |
| 15 | pointer-events fix | Children blocking clicks | ❌ Failed |
| 16 | Simplify button structure | Remove SVG/nested elements | ❌ Failed |
| 17 | Web research solutions | Apply macOS/WebView fixes | ❌ Failed |
| 18 | Multi-event handlers + z-index | Test all event types | ❌ Failed - ZERO events detected |
| 19 | useCallback memoization + button simplification | React handler recreation issue | ❌ Failed |

**Critical Discovery (Attempt #13)**: Button works with `btn.click()` in console but NOT with physical mouse clicks.

**Critical Discovery (Attempt #18)**: ZERO mouse/pointer events reach button - not onClick, not onMouseDown, not onPointerDown - NOTHING. Events intercepted before reaching React.

**Status**: The root cause remains unidentified after 19 attempts. The button is fully functional programmatically but completely unresponsive to physical mouse interaction. ALL event types (click, mousedown, mouseup, pointerdown, pointerup) are being blocked.
