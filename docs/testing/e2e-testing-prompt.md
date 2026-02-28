# Tally Agentic Wallet -- Exhaustive E2E Testing Prompt

> **Purpose**: This document is a self-contained prompt for a Claude Code agent (or subagent) to step through every screen and flow of the Tally Agentic Wallet app using Chrome browser automation and Gmail MCP tools. The agent executing this prompt should be able to run through all tests completely autonomously.

---

## Table of Contents

1. [Preamble](#1-preamble)
2. [Pre-flight Checklist](#2-pre-flight-checklist)
3. [Suite 1: Onboarding Flow](#suite-1-onboarding-flow)
4. [Suite 2: Install Skill](#suite-2-install-skill)
5. [Suite 3: Auth Flow -- Connect Coinbase](#suite-3-auth-flow----connect-coinbase)
6. [Suite 4: Auth Flow -- Verify OTP](#suite-4-auth-flow----verify-otp)
7. [Suite 5: Home Dashboard](#suite-5-home-dashboard)
8. [Suite 6: Add Funds](#suite-6-add-funds)
9. [Suite 7: Agents List](#suite-7-agents-list)
10. [Suite 8: Agent Detail](#suite-8-agent-detail)
11. [Suite 9: Transaction Detail](#suite-9-transaction-detail)
12. [Suite 10: Settings](#suite-10-settings)
13. [Suite 11: Navigation & Routing](#suite-11-navigation--routing)
14. [Suite 12: Visual Regression](#suite-12-visual-regression)
15. [Suite 13: Known Issues Verification](#suite-13-known-issues-verification)
16. [Gmail OTP Retrieval Flow](#gmail-otp-retrieval-flow)
17. [GIF Recording Instructions](#gif-recording-instructions)
18. [Teardown Checklist](#teardown-checklist)
19. [Test Results Template](#test-results-template)

---

## 1. Preamble

### Who You Are

You are a Claude Code agent tasked with performing exhaustive end-to-end testing of the Tally Agentic Wallet application. You will interact with the app through a Chrome browser using MCP-based browser automation tools, and you will use Gmail MCP tools to retrieve OTP codes for authentication.

### What Tally Agentic Wallet Is

Tally Agentic Wallet is a Tauri v2 desktop app (Rust backend + React frontend) for managing AI agent wallets. During development, the frontend is served by Vite at `http://localhost:1420`. The app is designed for a **390x844px** mobile-style window.

Key facts:
- **Auth email**: `dennison@dennisonbertram.com`
- **Wallet address**: `0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4`
- **All displayed data** comes from `src/data/placeholder_data.json` (balances, agents, transactions are fake)
- **Backend Tauri commands** are not available in the browser; the frontend catches errors and falls back gracefully
- **The user never interacts with CLI/terminal** -- this is a GUI-only app

### Available Tools

You have access to the following tool categories. **Always use `ToolSearch` to load MCP tools before calling them.**

#### Chrome Browser Automation (`mcp__claude-in-chrome__*`)

Load these with `ToolSearch` using query `"chrome"` or `"select:mcp__claude-in-chrome__navigate"` etc.

| Tool | Purpose |
|------|---------|
| `navigate` | Navigate to a URL |
| `read_page` | Read page content (text, links, interactive elements) |
| `computer` | Click, type, scroll, take screenshots |
| `get_page_text` | Get full text content of the page |
| `javascript_tool` | Execute JavaScript in the page context |
| `resize_window` | Resize browser window |
| `tabs_create_mcp` | Create a new browser tab |
| `tabs_context_mcp` | Get info about current tabs |
| `gif_creator` | Record a GIF of interactions |
| `find` | Find elements on the page |
| `form_input` | Fill form fields |
| `get_screenshot` | Take a screenshot |
| `upload_image` | Upload an image |

#### Gmail MCP (`mcp__gmail__*`)

Load these with `ToolSearch` using query `"gmail"`.

| Tool | Purpose |
|------|---------|
| `searchMessages` | Search Gmail for messages matching a query |
| `getMessage` | Get full message content by ID |
| `getThread` | Get entire email thread |
| `listMessages` | List messages in inbox |

#### Standard Claude Code Tools

- `Bash` -- Run shell commands
- `Read` / `Edit` / `Write` -- File operations
- `Grep` / `Glob` -- Search codebase

### General Testing Rules

1. **Always load tools before using them.** Call `ToolSearch` with the appropriate query before any MCP tool call.
2. **Take a screenshot after every significant action** to verify the result visually.
3. **Record test results** in the template at the end of this document.
4. **Do not modify source code** during testing unless explicitly instructed (e.g., auth bypass).
5. **Create a fresh tab** for testing -- do not reuse existing browser tabs.
6. **The app viewport is 390x844px.** Resize the browser window to this size before testing begins.

---

## 2. Pre-flight Checklist

Execute these steps before running any test suite.

### PF-001: Load Chrome Browser Tools

```
Action: Call ToolSearch with query "chrome navigate screenshot"
Expected: Chrome MCP tools are loaded and available
```

### PF-002: Load Gmail Tools

```
Action: Call ToolSearch with query "gmail search message"
Expected: Gmail MCP tools are loaded and available
```

### PF-003: Check Port Availability

```
Action: Run Bash command: lsof -i :1420
Expected: Either shows Vite dev server running, or no process (port free)
Decision:
  - If Vite is already running on 1420 -> proceed to PF-005
  - If nothing is running -> proceed to PF-004
  - If something ELSE is running on 1420 -> check ports 1421, 5173, 3000 with `lsof`; note the actual port
```

### PF-004: Start Dev Server (if not running)

```
Action: Run Bash command (background):
  cd /Users/dennisonbertram/Develop/apps/agent-neo-bank && npm run dev
Wait: 5 seconds, then verify with: lsof -i :1420
Expected: Vite dev server is running on port 1420
Note: If port differs, record the actual port and use it throughout all tests
```

### PF-005: Create Test Tab

```
Action: Call mcp__claude-in-chrome__tabs_create_mcp
  url: "http://localhost:1420"
Expected: New tab opens with the app loaded
```

### PF-006: Resize Window

```
Action: Call mcp__claude-in-chrome__resize_window
  width: 390
  height: 844
Expected: Browser window is resized to mobile dimensions
```

### PF-007: Verify App Loads

```
Action: Call mcp__claude-in-chrome__get_screenshot
Expected: Screenshot shows the Onboarding screen (first slide: "Your Agents, Your Rules")
Decision:
  - If blank white page -> check console for errors with mcp__claude-in-chrome__read_console_messages
  - If error page -> check dev server is running, correct port
  - If shows a different page -> app may have cached auth state; proceed with testing from wherever it landed
```

### PF-008: Verify Gmail Access

```
Action: Call mcp__gmail__searchMessages with query: "from:noreply@coinbase.com" and maxResults: 1
Expected: Returns results (or empty array if no prior emails) without authentication errors
Decision:
  - If authentication error -> Gmail MCP is not configured; skip Suite 4 (OTP verification) and note as BLOCKED
  - If success -> Gmail is accessible; proceed
```

---

## Suite 1: Onboarding Flow

**Route**: `/onboarding`
**Component**: `src/pages/Onboarding.tsx`
**Prerequisites**: App loaded on the onboarding page. If not on `/onboarding`, navigate there.

### ONB-001: First Slide Content

```
Test ID: ONB-001
Description: Verify the first onboarding slide displays correct content
Preconditions: App is on /onboarding
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/onboarding"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Title text: "Your Agents, Your Rules"
  - Description: "Set spending limits, approve transactions, and keep your AI agents accountable -- all from one dashboard."
  - Button text: "Next"
  - 4 indicator dots visible, first one is elongated/active (24px wide)
  - App logo visible at top
Verify: Screenshot shows centered content with logo, title, description, dots, and button
```

### ONB-002: Navigate to Second Slide

```
Test ID: ONB-002
Description: Clicking "Next" advances to the second slide
Preconditions: On first slide of onboarding
Steps:
  1. Call mcp__claude-in-chrome__find with query "Next button"
  2. Click the "Next" button using mcp__claude-in-chrome__computer with action "click" at the button coordinates
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Title: "Smart Spending Policies"
  - Description: "Define daily, weekly, and per-transaction limits. Agents operate within your boundaries automatically."
  - Button text: "Next"
  - Second indicator dot is now elongated/active
Verify: Screenshot shows second slide content, second dot active
```

### ONB-003: Navigate to Third Slide

```
Test ID: ONB-003
Description: Clicking "Next" advances to the third slide
Preconditions: On second slide
Steps:
  1. Click the "Next" button
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Title: "Real-Time Transparency"
  - Description: "Every transaction includes metadata -- what was purchased, why, and for which service. Full audit trail."
  - Button text: "Next"
  - Third indicator dot is active
Verify: Screenshot confirms third slide
```

### ONB-004: Navigate to Fourth (Final) Slide

```
Test ID: ONB-004
Description: Clicking "Next" advances to the fourth and final slide
Preconditions: On third slide
Steps:
  1. Click the "Next" button
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Title: "Get Started in Minutes"
  - Description: "Connect your Coinbase wallet, set up your first agent, and start managing AI spending today."
  - Button text: "Get set up" (NOT "Next")
  - Fourth indicator dot is active
Verify: Screenshot confirms final slide with "Get set up" CTA
```

### ONB-005: "Get set up" Navigates to Install Skill

```
Test ID: ONB-005
Description: Clicking "Get set up" on the final slide navigates to /setup/install
Preconditions: On fourth slide
Steps:
  1. Click the "Get set up" button
  2. Wait 500ms for navigation
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/setup/install"
  - Screen shows "Install Research Skill" content
Verify: URL check and screenshot confirm navigation to InstallSkill page
```

### ONB-006: Indicator Dot Animation

```
Test ID: ONB-006
Description: Verify indicator dots have proper styling -- active dot is wide, inactive dots are small circles
Preconditions: On onboarding page, any slide
Steps:
  1. Navigate to /onboarding
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('.flex.items-center.gap-\\[6px\\] > div')).map((d,i) => ({index: i, width: getComputedStyle(d).width, height: getComputedStyle(d).height, borderRadius: getComputedStyle(d).borderRadius})))`
Expected Result:
  - First dot: width ~24px, height ~6px (elongated, active)
  - Remaining 3 dots: width ~6px, height ~6px (small circles)
Verify: JavaScript output confirms correct dimensions
```

### ONB-007: Slide Transition Animation

```
Test ID: ONB-007
Description: Verify slide content has an animation class applied on transition
Preconditions: On onboarding first slide
Steps:
  1. Navigate to /onboarding
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('.animate-slide-up')?.className`
Expected Result:
  - An element with class "animate-slide-up" exists
Verify: Class presence confirms animation is applied
```

---

## Suite 2: Install Skill

**Route**: `/setup/install`
**Component**: `src/pages/InstallSkill.tsx`
**Prerequisites**: Navigate to `/setup/install`

### ISK-001: Install Screen Initial State

```
Test ID: ISK-001
Description: Verify the Install Skill screen shows correct initial content
Preconditions: None
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/setup/install"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Green icon badge with package icon visible
  - Title: "Install Research Skill"
  - Description about AI agents interacting with wallet
  - "What changes?" expandable section (collapsed by default)
  - "Confirm Installation" button (primary, black)
  - "Cancel" button (outline)
  - Footer text: "All changes are local to your machine."
Verify: Screenshot and text confirm all elements present
```

### ISK-002: Expand "What changes?" Section

```
Test ID: ISK-002
Description: Clicking "What changes?" expands to show file details
Preconditions: On Install Skill screen, section collapsed
Steps:
  1. Call mcp__claude-in-chrome__find with query "What changes?"
  2. Click on "What changes?" button
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Expanded section shows:
    - "claude.md" with "Config update" label and "UPDATED" badge
    - "agents.md" with "Permissions" label and "UPDATED" badge
  - ChevronDown icon rotates (from -90deg to 0deg)
Verify: Screenshot shows expanded section with both file items
```

### ISK-003: Collapse "What changes?" Section

```
Test ID: ISK-003
Description: Clicking "What changes?" again collapses the section
Preconditions: On Install Skill screen, section expanded
Steps:
  1. Click on "What changes?" button again
  2. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Section is collapsed, file details hidden
  - Only "What changes?" text and chevron visible
Verify: Screenshot shows collapsed state
```

### ISK-004: Confirm Installation Transitions to Success

```
Test ID: ISK-004
Description: Clicking "Confirm Installation" shows the success state
Preconditions: On Install Skill screen, initial state
Steps:
  1. Call mcp__claude-in-chrome__find with query "Confirm Installation"
  2. Click "Confirm Installation" button
  3. Wait 500ms
  4. Call mcp__claude-in-chrome__get_screenshot
  5. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Screen transitions to success state
  - Animated success checkmark visible (SuccessCheck component)
  - Title: "Skill installed"
  - Description mentions Research Skill configured and ready
  - "Continue" button visible
Verify: Screenshot shows success state with checkmark
Known Issue: #2 -- install_skill command does not exist; the screen fakes success with setState
```

### ISK-005: Continue from Success Navigates to Connect

```
Test ID: ISK-005
Description: Clicking "Continue" on success screen navigates to /setup/connect
Preconditions: On Install Skill success state
Steps:
  1. Click "Continue" button
  2. Wait 500ms
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
  4. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - URL pathname is "/setup/connect"
  - Screen shows "Connect your Coinbase account"
Verify: URL and screenshot confirm navigation
```

### ISK-006: Cancel Button Goes Back

```
Test ID: ISK-006
Description: Clicking "Cancel" navigates back to the previous page
Preconditions: On Install Skill screen (navigate fresh from onboarding)
Steps:
  1. Navigate to /onboarding, click through to last slide, click "Get set up"
  2. On Install Skill screen, click "Cancel"
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - Navigates back (to /onboarding)
Verify: URL confirms back navigation
```

---

## Suite 3: Auth Flow -- Connect Coinbase

**Route**: `/setup/connect`
**Component**: `src/pages/ConnectCoinbase.tsx`
**Prerequisites**: Navigate to `/setup/connect`

### CON-001: Connect Screen Initial State

```
Test ID: CON-001
Description: Verify the Connect Coinbase screen shows correct initial content
Preconditions: None
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/setup/connect"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Back button (circular, with ChevronLeft icon) at top
  - Title: "Connect your Coinbase account" (large, multiline)
  - Subtitle about linking wallet for agent operations
  - Email input field with label "EMAIL ADDRESS" and placeholder "name@email.com"
  - "Send code" button (disabled since email is empty)
  - Footer text: "A secure login link will be sent to your inbox."
  - Trust badge: blue Coinbase circle with "Secured by Coinbase Cloud"
Verify: Screenshot and text confirm all elements
```

### CON-002: Send Code Button Disabled When Empty

```
Test ID: CON-002
Description: The "Send code" button is disabled when email input is empty
Preconditions: On Connect screen, email field empty
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('button:has(> *)').disabled || Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Send code'))?.disabled`
  2. OR: Call mcp__claude-in-chrome__find with query "Send code button"
Expected Result:
  - "Send code" button has disabled attribute or disabled state (opacity-50)
Verify: Button is visually dimmed and non-interactive
```

### CON-003: Type Email Address

```
Test ID: CON-003
Description: Typing an email address enables the "Send code" button
Preconditions: On Connect screen
Steps:
  1. Call mcp__claude-in-chrome__find with query "email input"
  2. Click on the email input field
  3. Call mcp__claude-in-chrome__computer with action "type" and text "dennison@dennisonbertram.com"
  4. Call mcp__claude-in-chrome__get_screenshot
  5. Call mcp__claude-in-chrome__javascript_tool with code:
     `Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Send code'))?.disabled`
Expected Result:
  - Email appears in the input field
  - "Send code" button is now enabled (disabled === false)
Verify: Screenshot shows email in field, button is active
```

### CON-004: Send Code Triggers Auth Flow

```
Test ID: CON-004
Description: Clicking "Send code" with valid email triggers the auth login flow
Preconditions: Email "dennison@dennisonbertram.com" entered in field
Steps:
  1. Click the "Send code" button
  2. Call mcp__claude-in-chrome__get_screenshot immediately (to catch loading state)
  3. Wait 2 seconds
  4. Call mcp__claude-in-chrome__get_screenshot again
  5. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result (in browser without Tauri):
  - Button may briefly show "Sending..." text
  - Since Tauri invoke will fail in browser, an error message may appear:
    "Failed to send code. Please try again." or similar
  - OR if the error is caught and navigation happens anyway, URL becomes /setup/verify
Note: In browser (non-Tauri) the tauriApi.auth.login call will throw since `invoke` is not available.
  The component catches this and shows the error. This is expected behavior for browser testing.
  For real OTP testing, see Suite 4 and the Gmail OTP Retrieval Flow section.
Verify: Screenshot shows either error state or navigation to verify page
```

### CON-005: Back Button Navigation

```
Test ID: CON-005
Description: Back button navigates to the previous page
Preconditions: On Connect screen
Steps:
  1. Navigate to /setup/connect
  2. Find and click the back button (circular button with chevron icon at top)
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - Navigates back in history
Verify: URL shows previous route
```

### CON-006: Enter Key Submits Form

```
Test ID: CON-006
Description: Pressing Enter in the email field triggers the send code action
Preconditions: On Connect screen with email entered
Steps:
  1. Navigate to /setup/connect
  2. Click email input and type "test@example.com"
  3. Call mcp__claude-in-chrome__computer with action "key" and key "Return"
  4. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Same behavior as clicking "Send code" -- loading state or error shown
Verify: Screenshot shows loading or error state (not still idle)
```

### CON-007: Error State Display

```
Test ID: CON-007
Description: When auth_login fails, an error message is displayed
Preconditions: On Connect screen in browser (non-Tauri, so invoke will fail)
Steps:
  1. Navigate to /setup/connect
  2. Type email and click "Send code"
  3. Wait for error
  4. Call mcp__claude-in-chrome__get_page_text
  5. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Red error text appears below the input: "Failed to send code. Please try again." or the actual error message
  - Error text uses the danger color (var(--color-danger))
Verify: Error text is visible in screenshot and page text
```

---

## Suite 4: Auth Flow -- Verify OTP

**Route**: `/setup/verify`
**Component**: `src/pages/VerifyOtp.tsx`
**Prerequisites**: Must have completed a real auth flow (sent OTP email) OR navigate directly to test UI

> **IMPORTANT**: This suite requires real Tauri backend to send OTP emails. In browser-only testing, you can test the UI by navigating directly to `/setup/verify`. For full OTP verification, see the [Gmail OTP Retrieval Flow](#gmail-otp-retrieval-flow) section.

### OTP-001: Verify Screen Initial State

```
Test ID: OTP-001
Description: Verify OTP screen shows correct initial content
Preconditions: Navigate to /setup/verify (may need to pass state)
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/setup/verify"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Back button at top (circular with ChevronLeft)
  - Title: "Verify it's you"
  - Subtitle: "Enter the 6-digit code sent to" followed by email (may be empty if no state passed)
  - 6 OTP input boxes visible (each 48x56px, rounded-12px, bg-secondary)
  - "Verify code" button (disabled since OTP not entered)
  - "Resend code" text with countdown timer "in 0:42"
Verify: Screenshot shows all elements
Known Issue: #1 -- OTP input boxes may not be visible; check if they render
```

### OTP-002: OTP Input Boxes Visibility Check

```
Test ID: OTP-002
Description: Verify OTP input boxes are visible and styled correctly
Preconditions: On verify screen
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('input[inputmode="numeric"]')).map((el, i) => ({
       index: i,
       width: el.offsetWidth,
       height: el.offsetHeight,
       visible: el.offsetWidth > 0 && el.offsetHeight > 0,
       bgColor: getComputedStyle(el).backgroundColor
     })))`
Expected Result:
  - 6 input elements found
  - Each has width ~48px and height ~56px
  - All are visible (offsetWidth > 0 and offsetHeight > 0)
  - Background color matches --bg-secondary
Decision:
  - If inputs have 0 width/height -> FAIL, matches Known Issue #1
  - If inputs are visible -> PASS
Verify: JavaScript output confirms dimensions
Known Issue: #1 -- OTP input boxes not visible
```

### OTP-003: Type OTP Digits

```
Test ID: OTP-003
Description: Typing digits in OTP fields advances focus to next field
Preconditions: On verify screen, OTP boxes visible
Steps:
  1. Click on the first OTP input box
  2. Type "1" using mcp__claude-in-chrome__computer action "type"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Type "2"
  5. Type "3"
  6. Type "4"
  7. Type "5"
  8. Call mcp__claude-in-chrome__get_screenshot
  9. Type "6"
  10. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Each digit appears in its respective box
  - Focus automatically advances to the next box after entering a digit
  - After entering all 6 digits, the onComplete callback fires (may auto-submit)
  - "Verify code" button becomes enabled
Verify: Screenshots show progressive digit entry
```

### OTP-004: Backspace Deletes and Moves Focus Back

```
Test ID: OTP-004
Description: Pressing backspace in an empty OTP field moves focus to the previous field
Preconditions: On verify screen with some digits entered
Steps:
  1. Navigate fresh to /setup/verify
  2. Click first input, type "123"
  3. Press Backspace key
  4. Call mcp__claude-in-chrome__get_screenshot
  5. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.activeElement?.closest('input')?.previousElementSibling ? 'moved back' : 'still on same'`
Expected Result:
  - Backspace in empty field moves focus to previous field and clears it
  - Digit "3" is removed, focus is on the second input
Verify: Screenshot shows cursor in earlier box
```

### OTP-005: Countdown Timer

```
Test ID: OTP-005
Description: Countdown timer starts at 0:42 and counts down
Preconditions: On verify screen, freshly loaded
Steps:
  1. Navigate to /setup/verify
  2. Call mcp__claude-in-chrome__get_page_text (note countdown value)
  3. Wait 3 seconds (Bash: sleep 3)
  4. Call mcp__claude-in-chrome__get_page_text (note new countdown value)
Expected Result:
  - First read shows "in 0:42" or similar
  - Second read shows "in 0:39" or similar (3 seconds less)
  - "Resend code" button is disabled while countdown > 0
Verify: Text comparison confirms countdown is decrementing
```

### OTP-006: Resend Code Disabled During Countdown

```
Test ID: OTP-006
Description: "Resend code" button is disabled while countdown is active
Preconditions: On verify screen with countdown > 0
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Resend'))?.disabled`
Expected Result:
  - Returns true (button is disabled)
  - Button text color is tertiary (dimmed)
Verify: JavaScript confirms disabled state
```

### OTP-007: Verify Code Button Disabled Without Full OTP

```
Test ID: OTP-007
Description: "Verify code" button is disabled when fewer than 6 digits entered
Preconditions: On verify screen
Steps:
  1. Navigate to /setup/verify
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Verify'))?.disabled`
Expected Result:
  - Returns true (disabled because no digits entered)
Verify: JavaScript confirms disabled
```

### OTP-008: Full OTP Flow with Gmail (Real Auth)

```
Test ID: OTP-008
Description: Complete OTP verification using real email from Coinbase
Preconditions:
  - Tauri dev server running (not just Vite -- needs `cargo tauri dev`)
  - Gmail MCP tools loaded and working
  - PF-008 passed (Gmail accessible)
Steps:
  SEE: "Gmail OTP Retrieval Flow" section below for detailed steps
Expected Result:
  - OTP is retrieved from Gmail
  - Code is entered into OTP fields
  - Verification succeeds
  - User is redirected to /home
Decision:
  - If not running in Tauri -> SKIP this test, mark as "SKIPPED -- browser only"
  - If Gmail not accessible -> SKIP, mark as "BLOCKED -- no Gmail"
Verify: URL changes to /home after verification
```

---

## Suite 5: Home Dashboard

**Route**: `/home`
**Component**: `src/pages/Home.tsx`
**Prerequisites**: Must be authenticated. Use the auth bypass method below.

### Auth Bypass for Protected Routes

To test protected routes without going through the real auth flow, use one of these methods:

**Method A: JavaScript injection (preferred, non-destructive)**
```
Steps:
  1. Navigate to any page (e.g., /onboarding)
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `(() => {
       // Access Zustand store and set authenticated
       const store = window.__zustand_stores?.auth;
       if (store) { store.setState({ isAuthenticated: true, email: 'dennison@dennisonbertram.com' }); }
       // Alternative: use React internals
       return 'attempted auth bypass';
     })()`
  3. Navigate to /home
  4. If redirected to /onboarding, Method A failed -- use Method B
```

**Method B: Source code edit (temporary, must revert)**
```
Steps:
  1. Edit /Users/dennisonbertram/Develop/apps/agent-neo-bank/src/stores/authStore.ts
  2. Change line 16 from `isAuthenticated: false` to `isAuthenticated: true`
  3. Save -- Vite HMR will reload the app
  4. Navigate to /home
  5. IMPORTANT: Revert this change after testing! (see Teardown Checklist)
```

**Method C: Direct navigation with localStorage hack**
```
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `(() => {
       // Try to find and update the Zustand persisted state if any
       // Since this store is not persisted, we need to hook into the module
       // Navigate and check if the non-Tauri fallback preserves state
       return window.location.pathname;
     })()`
  2. If on /onboarding, the auth check redirected. Use Method B.
```

### HOM-001: Home Screen Layout

```
Test ID: HOM-001
Description: Verify the home screen shows all major sections
Preconditions: Authenticated (use auth bypass), on /home
Steps:
  1. Ensure authenticated state (see Auth Bypass above)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/home"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Top bar: "Tally Wallet" text and user initials avatar "DB"
  - Balance card (black background, rounded-32px):
    - "Base Network Balance" label
    - "$20.32" total balance
    - "BASE" badge with blue dot
    - Truncated wallet address: "0x72AE...504B4"
    - "0.10 ETH" and "20.00 USDC" breakdown
  - Action buttons: "Add Funds" and "Agents" (side by side)
  - Segment control: "Overview" (active) | "Agents"
  - Activity feed with "Activity" header and "View All" link
  - Transaction list items
  - Bottom navigation bar with 5 items: Home, Agents, [+], Stats, Settings
Verify: Screenshot shows all sections, text confirms data values
```

### HOM-002: Balance Card Data

```
Test ID: HOM-002
Description: Verify balance card shows correct placeholder data
Preconditions: On /home
Steps:
  1. Call mcp__claude-in-chrome__get_page_text
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('.bg-black.text-white')?.textContent`
Expected Result:
  - Total balance: "$20.32"
  - Address: "0x72AE...504B4"
  - ETH: "0.10 ETH"
  - USDC: "20.00 USDC"
  - Network: "BASE"
Verify: Text extraction matches placeholder_data.json values
Known Issue: #3 -- get_balance and get_address are stubs; values come from placeholder_data.json
```

### HOM-003: Add Funds Button Navigation

```
Test ID: HOM-003
Description: Clicking "Add Funds" navigates to /add-funds
Preconditions: On /home, authenticated
Steps:
  1. Call mcp__claude-in-chrome__find with query "Add Funds button"
  2. Click "Add Funds" button
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/add-funds"
Verify: URL confirms navigation
```

### HOM-004: Agents Button Navigation

```
Test ID: HOM-004
Description: Clicking "Agents" button navigates to /agents
Preconditions: On /home, authenticated
Steps:
  1. Navigate to /home
  2. Call mcp__claude-in-chrome__find with query "Agents button"
  3. Click "Agents" button
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/agents"
Verify: URL confirms navigation
```

### HOM-005: Segment Control -- Overview Tab

```
Test ID: HOM-005
Description: "Overview" segment shows the activity feed
Preconditions: On /home, Overview segment selected (default)
Steps:
  1. Navigate to /home
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - "Activity" section header visible
  - "View All" link visible
  - Transaction items listed:
    - "Research Agent" / "OpenAI GPT-4 API call" / "-2.50 USDC" / "API FEE"
    - "Deploy Bot" / "Contract deployment gas" / "-0.0034 ETH" / "GAS"
    - "Research Agent" / "Anthropic Claude API call" / "-1.00 USDC" / "API FEE"
    - "Treasury" / "USDC to ETH swap via Uniswap" / "-5.00 USDC" / "SWAP"
    - "Deposit" / "Funding deposit" / "+10.00 USDC" / "DEPOSIT"
Verify: All 5 transactions visible with correct data
```

### HOM-006: Segment Control -- Agents Tab

```
Test ID: HOM-006
Description: Switching to "Agents" segment shows agent pill rows
Preconditions: On /home
Steps:
  1. Call mcp__claude-in-chrome__find with query "Agents" (in segment control, not the action button)
  2. Click the "Agents" segment tab (the one inside the segment control, not the action button)
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Activity feed is replaced by agent pill rows:
    - "Research" / "$3.50" / "today" (green accent #8FB5AA)
    - "Deploy Bot" / "$0.01" / "today" (yellow accent #F2D48C)
    - "Treasury" / "$0.00" / "paused" (terracotta accent #D9A58B)
Verify: Screenshot shows 3 agent pill rows
```

### HOM-007: Transaction Item Click

```
Test ID: HOM-007
Description: Clicking a transaction navigates to its detail page
Preconditions: On /home, Overview segment
Steps:
  1. Navigate to /home
  2. Call mcp__claude-in-chrome__find with query "OpenAI GPT-4 API call"
  3. Click on the first transaction item (Research Agent / OpenAI)
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/transactions/tx-001"
Verify: URL confirms navigation to transaction detail
```

### HOM-008: Bottom Navigation Bar

```
Test ID: HOM-008
Description: Bottom nav shows 5 items with Home highlighted
Preconditions: On /home
Steps:
  1. Navigate to /home
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('nav button')).map(b => ({
       text: b.textContent.trim(),
       isActive: !b.className.includes('text-secondary') || b.className.includes('bg-black')
     })))`
Expected Result:
  - 5 buttons: "Home", "Agents", "" (FAB/plus), "Stats", "Settings"
  - Home is active (not using text-secondary color)
  - FAB button is a black circle with plus icon, elevated above the nav bar
Verify: JavaScript output shows 5 nav items with Home active
```

### HOM-009: Scroll Behavior

```
Test ID: HOM-009
Description: Content scrolls properly, top bar remains sticky
Preconditions: On /home with enough content to scroll
Steps:
  1. Navigate to /home
  2. Call mcp__claude-in-chrome__computer with action "scroll" and direction "down" by 300px
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('.sticky.top-0')?.getBoundingClientRect().top`
Expected Result:
  - Content scrolls up
  - Top bar remains at top of viewport (sticky position, top ~0)
  - Bottom nav remains fixed at bottom
Verify: Screenshot shows scrolled content, sticky header
```

---

## Suite 6: Add Funds

**Route**: `/add-funds`
**Component**: `src/pages/AddFunds.tsx`
**Prerequisites**: Authenticated, navigate to `/add-funds`

### AFD-001: Add Funds Screen Layout

```
Test ID: AFD-001
Description: Verify Add Funds screen shows all elements
Preconditions: Authenticated, on /add-funds
Steps:
  1. Ensure authenticated (auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/add-funds"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - QR code placeholder: 200x200px dashed-border box with Grid3x3 icon
  - Warning pill: yellow background, "Send only USDC or ETH on Base"
  - Wallet Address section:
    - Label: "WALLET ADDRESS" (uppercase)
    - Full address: "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4"
    - Copy button (circular, with Copy icon)
  - "Buy with Card (Coming Soon)" button -- disabled, opacity 50%
  - "Close" button (outline)
Verify: Screenshot shows all elements, QR is a placeholder
Known Issue: #6 -- QR code is a placeholder, not a real QR
```

### AFD-002: Copy Wallet Address

```
Test ID: AFD-002
Description: Clicking copy button copies address and shows check icon
Preconditions: On /add-funds
Steps:
  1. Call mcp__claude-in-chrome__find with query "copy button"
  2. Click the copy button (circular button next to address)
  3. Call mcp__claude-in-chrome__get_screenshot (immediately to catch the Check icon)
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `navigator.clipboard.readText()`
Expected Result:
  - Copy icon changes to a green Check icon for 2 seconds
  - Clipboard contains "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4"
Note: Clipboard read may be blocked by browser permissions. If so, verify visually via the icon change.
Verify: Screenshot shows check icon (green) instead of copy icon
```

### AFD-003: Buy with Card Button Disabled

```
Test ID: AFD-003
Description: "Buy with Card (Coming Soon)" button is disabled and dimmed
Preconditions: On /add-funds
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `(() => {
       const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Coming Soon'));
       return btn ? { disabled: btn.disabled, opacity: getComputedStyle(btn).opacity, text: btn.textContent } : null;
     })()`
Expected Result:
  - Button exists with text containing "Coming Soon"
  - disabled: true
  - opacity: "0.5" (or class includes opacity-50)
Verify: JavaScript confirms disabled state
Known Issue: #5 -- buy_with_card not implemented
```

### AFD-004: Close Button Returns to Home

```
Test ID: AFD-004
Description: Clicking "Close" navigates back to /home
Preconditions: On /add-funds
Steps:
  1. Call mcp__claude-in-chrome__find with query "Close button"
  2. Click "Close" button
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/home"
Verify: URL confirms navigation
```

---

## Suite 7: Agents List

**Route**: `/agents`
**Component**: `src/pages/AgentsList.tsx`
**Prerequisites**: Authenticated, navigate to `/agents`

### AGT-001: Agents List Screen Layout

```
Test ID: AGT-001
Description: Verify the agents list screen shows header, segment control, and agent cards
Preconditions: Authenticated, on /agents
Steps:
  1. Ensure authenticated (auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/agents"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Header: "My Agents" title
  - Settings icon button (gear) at top right
  - Segment control: "Active" | "All Agents" (default selected) | "Archived"
  - 3 agent cards displayed:
    1. Research Agent -- "Gathers market data and analyzes trends" -- Active badge -- $3.50/$25.00 progress bar
    2. Deploy Bot -- "Deploys and manages smart contracts" -- Active badge -- $0.0034/$0.05 progress bar
    3. Treasury -- "Manages portfolio rebalancing and yield" -- Pending badge -- $0.00/$50.00 progress bar
  - Bottom nav with "Agents" tab active
Verify: Screenshot shows all 3 agent cards with correct data
```

### AGT-002: Segment Control -- Active Filter

```
Test ID: AGT-002
Description: Selecting "Active" segment shows only active agents
Preconditions: On /agents
Steps:
  1. Call mcp__claude-in-chrome__find with query "Active" (segment control option)
  2. Click "Active" segment
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Only 2 agents shown: Research Agent and Deploy Bot (both have status "active")
  - Treasury (status "pending") is NOT shown
Verify: Screenshot shows exactly 2 agent cards
```

### AGT-003: Segment Control -- Archived Filter

```
Test ID: AGT-003
Description: Selecting "Archived" segment shows only revoked agents (or empty state)
Preconditions: On /agents
Steps:
  1. Click "Archived" segment
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Empty state message: "No agents found" / "No archived agents yet."
  - No agent cards visible (none have "revoked" status in placeholder data)
Verify: Screenshot shows empty state
```

### AGT-004: Segment Control -- All Agents (Reset)

```
Test ID: AGT-004
Description: Selecting "All Agents" shows all agents regardless of status
Preconditions: On /agents, "Archived" segment selected
Steps:
  1. Click "All Agents" segment
  2. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - All 3 agents visible again
Verify: Screenshot shows 3 agent cards
```

### AGT-005: Agent Card Click Navigates to Detail

```
Test ID: AGT-005
Description: Clicking an agent card navigates to the agent detail page
Preconditions: On /agents, all agents visible
Steps:
  1. Call mcp__claude-in-chrome__find with query "Research Agent"
  2. Click on the Research Agent card
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/agents/agent-research-001"
Verify: URL confirms navigation to agent detail
```

### AGT-006: Settings Icon Navigates to Settings

```
Test ID: AGT-006
Description: Clicking the settings gear icon navigates to /settings
Preconditions: On /agents
Steps:
  1. Navigate to /agents
  2. Find and click the settings icon button (gear icon at top right)
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/settings"
Verify: URL confirms navigation
```

### AGT-007: Agent Card Progress Bar

```
Test ID: AGT-007
Description: Agent cards show correct progress bars based on daily spend vs cap
Preconditions: On /agents, All Agents selected
Steps:
  1. Navigate to /agents
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('.bg-\\[var\\(--bg-secondary\\)\\].rounded-\\[20px\\]')).map(card => {
       const nameEl = card.querySelector('.text-\\[15px\\].font-semibold');
       const progressBar = card.querySelector('[style*="width"]');
       return {
         name: nameEl?.textContent,
         progressWidth: progressBar?.style.width
       };
     }))`
Expected Result:
  - Research Agent: progress ~14% (3.50/25.00)
  - Deploy Bot: progress ~6.8% (0.0034/0.05)
  - Treasury: progress 0% (0/50)
Verify: JavaScript shows approximate percentage widths
```

---

## Suite 8: Agent Detail

**Route**: `/agents/:agentId`
**Component**: `src/pages/AgentDetail.tsx`
**Prerequisites**: Authenticated, navigate to an agent detail page

### ADT-001: Agent Detail Screen Layout

```
Test ID: ADT-001
Description: Verify the agent detail screen shows all sections
Preconditions: Authenticated
Steps:
  1. Ensure authenticated (auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/agents/agent-research-001"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Sticky header with back button and status pill ("Active")
  - "Local Agent" label
  - Agent name: "Research Agent"
  - Description: "Gathers market data and analyzes trends"
  - Daily Spend card:
    - "Daily Spend" label
    - "$3.50 / $25.00" spend vs limit
    - Progress bar (green accent, ~14% filled)
    - "14% Used" label
    - "Reset in 14h" label
    - Toggle switch (for pause/resume)
  - Spending Controls section:
    - "Daily Limit" stepper ($25.00 default)
    - "Per Transaction" stepper ($5.00 default)
    - "Approval Threshold" toggle with description
  - Agent History section:
    - "Filter" link
    - 3 hardcoded history items: Arxiv API Call, Cross-Chain Query, Metadata Storage
  - "Save Changes" button at bottom
Verify: Screenshot shows all sections
```

### ADT-002: Back Button Navigation

```
Test ID: ADT-002
Description: Back button navigates to previous page
Preconditions: Navigated to agent detail from agents list
Steps:
  1. Navigate to /agents first, then click Research Agent
  2. On agent detail, click back button
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/agents"
Verify: URL confirms back navigation
```

### ADT-003: Toggle Pause/Resume

```
Test ID: ADT-003
Description: Toggle switch changes agent status between active and paused
Preconditions: On agent detail for Research Agent (agent-research-001)
Steps:
  1. Navigate to /agents/agent-research-001
  2. Note initial toggle state -- Research Agent is "active" so toggle should be unchecked (isPaused = false for "active")
  3. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('[role="switch"]')?.getAttribute('aria-checked')`
  4. Click the toggle switch in the Daily Spend card
  5. Call mcp__claude-in-chrome__get_screenshot
  6. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('[role="switch"]')?.getAttribute('aria-checked')`
Expected Result:
  - Initial: aria-checked is "false" (agent is active, isPaused = false)
  - After click: aria-checked is "true" (agent is paused)
  - Status pill in header changes to "Paused"
  - Progress bar opacity reduces to 0.3
Verify: Screenshot shows paused state, JavaScript confirms toggle change
Known Issue: #7 -- resume_agent command does not exist; toggle only changes local UI state
```

### ADT-004: Daily Limit Stepper -- Increase

```
Test ID: ADT-004
Description: Clicking the plus button on Daily Limit increases the value by $5
Preconditions: On agent detail
Steps:
  1. Navigate to /agents/agent-research-001
  2. Find the Daily Limit stepper row
  3. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('.min-w-\\[60px\\]')[0]?.textContent`
  4. Note initial value ($25.00)
  5. Find and click the plus button next to the daily limit value
  6. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('.min-w-\\[60px\\]')[0]?.textContent`
Expected Result:
  - Initial: "$25.00"
  - After click: "$30.00" (increased by step of 5)
Verify: JavaScript confirms value change
```

### ADT-005: Daily Limit Stepper -- Decrease

```
Test ID: ADT-005
Description: Clicking the minus button on Daily Limit decreases the value by $5
Preconditions: On agent detail, daily limit at $30 (from ADT-004)
Steps:
  1. Find and click the minus button next to the daily limit value
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('.min-w-\\[60px\\]')[0]?.textContent`
Expected Result:
  - Value returns to "$25.00"
Verify: JavaScript confirms value change
```

### ADT-006: Per Transaction Stepper

```
Test ID: ADT-006
Description: Per Transaction stepper increments/decrements by $1
Preconditions: On agent detail
Steps:
  1. Navigate to /agents/agent-research-001
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('.min-w-\\[60px\\]')[1]?.textContent`
  3. Note initial value ($5.00)
  4. Click the plus button for Per Transaction
  5. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('.min-w-\\[60px\\]')[1]?.textContent`
Expected Result:
  - Initial: "$5.00"
  - After increment: "$6.00"
Verify: JavaScript confirms value
```

### ADT-007: Approval Threshold Toggle

```
Test ID: ADT-007
Description: Approval Threshold toggle changes state and updates description
Preconditions: On agent detail
Steps:
  1. Navigate to /agents/agent-research-001
  2. Find all toggles on the page
  3. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('[role="switch"]')).map((t, i) => ({index: i, checked: t.getAttribute('aria-checked')})))`
  4. Note the approval threshold toggle (should be the last one, index 2 if 3 toggles exist, or index 1 if 2)
  5. Click the approval threshold toggle
  6. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Toggle state changes
  - Description text "Prompt for any tx > $5.00" updates based on current perTxLimit value
Verify: Screenshot and JavaScript confirm toggle state change
```

### ADT-008: Agent History Section

```
Test ID: ADT-008
Description: Agent history shows 3 hardcoded transactions
Preconditions: On agent detail
Steps:
  1. Navigate to /agents/agent-research-001
  2. Scroll down to Agent History section
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - "Agent History" header with "Filter" link
  - 3 items:
    1. "Arxiv API Call" / "Today, 2:45 PM" / "Success" / "-$1.20"
    2. "Cross-Chain Query" / "Today, 11:20 AM" / "Success" / "-$3.80"
    3. "Metadata Storage" / "Yesterday, 9:15 PM" / "Success" / "-$1.50"
Verify: Page text contains all 3 history items
```

### ADT-009: Save Changes Button

```
Test ID: ADT-009
Description: "Save Changes" button navigates back
Preconditions: On agent detail
Steps:
  1. Navigate to /agents/agent-research-001 (from /agents)
  2. Scroll down to "Save Changes" button
  3. Click "Save Changes"
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - Navigates back (to /agents or wherever the user came from)
Verify: URL confirms navigation
```

### ADT-010: Treasury Agent -- Pending Status

```
Test ID: ADT-010
Description: Treasury agent shows "Pending" status and different initial state
Preconditions: Authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/agents/agent-treasury-003"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Agent name: "Treasury"
  - Status pill shows "Paused" (because status is "pending", isPaused defaults to true)
  - Daily spend: "$0.00 / $50.00"
  - Progress bar at 0%
  - Toggle is checked (isPaused = true)
Verify: Screenshot shows paused treasury agent
```

---

## Suite 9: Transaction Detail

**Route**: `/transactions/:txId`
**Component**: `src/pages/TransactionDetail.tsx`
**Prerequisites**: Authenticated

### TXD-001: Transaction Detail -- Send Transaction

```
Test ID: TXD-001
Description: Verify transaction detail for a send transaction (tx-001)
Preconditions: Authenticated
Steps:
  1. Ensure authenticated (auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/transactions/tx-001"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Back nav: "< Details" link
  - Amount hero: "-2.50 USDC" (large display text)
  - Date/time: formatted from timestamp 1740700800 (e.g., "February 28, 2025")
  - Agent identity row:
    - Green square icon with Search icon
    - "Research Agent" name
    - "Verified Agent" badge
  - Agent Metadata card:
    - Category: "API Fee"
    - Purpose: "OpenAI GPT-4 API call"
    - Request ID: "REQ_X-001" or similar derived from tx.id
  - Cost Breakdown card:
    - Service Fee: "-2.50 USDC"
    - Network Fee: "$0.00"
  - Notes section: "Market analysis batch job" (from memo)
  - "View on Explorer" button
Verify: All sections present with correct data from placeholder_data.json
```

### TXD-002: Transaction Detail -- Receive Transaction

```
Test ID: TXD-002
Description: Verify transaction detail for a receive/deposit transaction (tx-005)
Preconditions: Authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/transactions/tx-005"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Amount: "+10.00 USDC" (positive amount)
  - No agent identity row (agent_name is null)
  - Metadata card:
    - Category: "Deposit"
    - Purpose: "Funding deposit"
  - Notes: "Initial funding"
Verify: Screenshot shows deposit transaction without agent info
```

### TXD-003: Transaction Not Found

```
Test ID: TXD-003
Description: Navigating to a non-existent transaction shows error
Preconditions: Authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/transactions/tx-999"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Text: "Transaction not found."
Verify: Error message displayed
```

### TXD-004: Back Navigation

```
Test ID: TXD-004
Description: "< Details" back link navigates to previous page
Preconditions: On transaction detail
Steps:
  1. Navigate to /home first, then click a transaction to go to /transactions/tx-001
  2. Click the "< Details" back link
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - Navigates back to /home
Verify: URL confirms back navigation
```

### TXD-005: View on Explorer Button

```
Test ID: TXD-005
Description: "View on Explorer" button opens BaseScan in new tab
Preconditions: On transaction detail
Steps:
  1. Navigate to /transactions/tx-001
  2. Call mcp__claude-in-chrome__find with query "View on Explorer"
  3. Note: Clicking will open a new tab to basescan.org
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Explorer'))?.onclick?.toString().includes('basescan') || 'has click handler'`
Expected Result:
  - Button exists with ExternalLink icon
  - Click handler opens https://basescan.org in a new tab
Note: Do not actually click in automated testing to avoid navigating away
Verify: JavaScript confirms the button and its handler
```

### TXD-006: Awaiting Approval Transaction

```
Test ID: TXD-006
Description: Verify transaction with "awaiting_approval" status (tx-004)
Preconditions: Authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/transactions/tx-004"
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Amount: "-5.00 USDC"
  - Agent: "Treasury"
  - Category: "Swap"
  - Purpose: "USDC to ETH swap via Uniswap"
  - Memo: "Portfolio rebalance"
Verify: All metadata correct for the awaiting approval transaction
```

---

## Suite 10: Settings

**Route**: `/settings`
**Component**: `src/pages/Settings.tsx`
**Prerequisites**: Authenticated

### SET-001: Settings Screen Layout

```
Test ID: SET-001
Description: Verify settings screen shows all sections
Preconditions: Authenticated
Steps:
  1. Ensure authenticated (auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/settings"
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - "Home" back button at top
  - Profile section:
    - Avatar circle with initials "DB" (terracotta background)
    - Name: "Dennison Bertram"
    - Email: "dennison@dennisonbertram.com"
  - Notifications section header
  - 5 notification toggle rows:
    1. "Agent Requests" / "New agent registration alerts" -- toggle ON
    2. "Transaction Completed" / "Confirmation when transactions settle" -- toggle OFF
    3. "Approval Required" / "When agents need spending approval" -- toggle ON
    4. "Daily Limit Reached" / "Alert when daily budget is exhausted" -- toggle OFF
    5. "Low Balance" / "Warning when wallet balance is low" -- toggle ON
  - Account & Security section:
    - "Reset Coinbase Connection" (red text) / "Disconnect and re-authenticate your wallet"
    - "Export Wallet History" / "Download CSV of all agent activity"
  - Version footer: "v0.1.0 (Base Mainnet)"
  - Bottom nav with Settings tab active
Verify: Screenshot shows all sections with correct toggle states
```

### SET-002: Notification Toggle -- Agent Requests

```
Test ID: SET-002
Description: Toggling "Agent Requests" changes its state
Preconditions: On /settings
Steps:
  1. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify(Array.from(document.querySelectorAll('[role="switch"]')).map((t, i) => ({index: i, checked: t.getAttribute('aria-checked')})))`
  2. Note "Agent Requests" toggle is at index 0, initially "true"
  3. Click the first toggle
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('[role="switch"]')[0]?.getAttribute('aria-checked')`
Expected Result:
  - Toggle changes from "true" to "false"
  - Visual state: track color changes from black to bg-secondary
Verify: JavaScript confirms state change
```

### SET-003: Notification Toggle -- Transaction Completed

```
Test ID: SET-003
Description: Toggling "Transaction Completed" changes its state
Preconditions: On /settings
Steps:
  1. Note "Transaction Completed" toggle is at index 1, initially "false"
  2. Click the second toggle
  3. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelectorAll('[role="switch"]')[1]?.getAttribute('aria-checked')`
Expected Result:
  - Toggle changes from "false" to "true"
Verify: JavaScript confirms state change
```

### SET-004: Home Button Navigation

```
Test ID: SET-004
Description: "Home" back button navigates to /home
Preconditions: On /settings
Steps:
  1. Navigate to /settings
  2. Call mcp__claude-in-chrome__find with query "Home" (the back button, not the nav)
  3. Click the "Home" button at the top
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/home"
Verify: URL confirms navigation
```

### SET-005: Reset Coinbase Connection

```
Test ID: SET-005
Description: "Reset Coinbase Connection" shows confirmation dialog
Preconditions: On /settings
Steps:
  1. Navigate to /settings
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `window.confirm = (msg) => { window.__lastConfirm = msg; return false; }; 'intercepted'`
  3. Click "Reset Coinbase Connection" button
  4. Call mcp__claude-in-chrome__javascript_tool with code: `window.__lastConfirm`
Expected Result:
  - Confirmation dialog message: "Are you sure you want to reset your Coinbase connection?"
  - When cancelled (we return false), user stays on /settings
Verify: JavaScript confirms the dialog message text
```

### SET-006: Reset Coinbase Connection -- Confirmed

```
Test ID: SET-006
Description: Confirming reset redirects to onboarding
Preconditions: On /settings
Steps:
  1. Navigate to /settings
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `window.confirm = () => true; 'intercepted with true'`
  3. Click "Reset Coinbase Connection" button
  4. Wait 500ms
  5. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding"
  - User is logged out and redirected
Verify: URL confirms redirect to onboarding
```

### SET-007: Export Wallet History

```
Test ID: SET-007
Description: "Export Wallet History" button exists but has no functional handler
Preconditions: On /settings
Steps:
  1. Navigate to /settings
  2. Call mcp__claude-in-chrome__find with query "Export Wallet History"
  3. Note: The button exists but has no onClick handler (or an empty one)
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `Array.from(document.querySelectorAll('button')).find(b => b.textContent.includes('Export'))?.onclick`
Expected Result:
  - Button is present in the DOM
  - No functional click handler (returns null or undefined)
Verify: JavaScript confirms button exists but has no handler
Known Issue: #4 -- export_wallet_history not implemented
```

### SET-008: Version Display

```
Test ID: SET-008
Description: Version string shows correct app version
Preconditions: On /settings
Steps:
  1. Call mcp__claude-in-chrome__get_page_text
Expected Result:
  - Text contains "v0.1.0 (Base Mainnet)"
Verify: Text search confirms version string
```

---

## Suite 11: Navigation & Routing

**Prerequisites**: Various states

### NAV-001: Default Route Redirect

```
Test ID: NAV-001
Description: Root URL redirects to /onboarding
Preconditions: Not authenticated
Steps:
  1. Make sure auth state is reset (isAuthenticated: false)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/"
  3. Wait 500ms
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding"
Verify: URL confirms redirect
```

### NAV-002: Unknown Route Redirect

```
Test ID: NAV-002
Description: Unknown routes redirect to /onboarding
Preconditions: Not authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/some-random-page"
  2. Wait 500ms
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding"
Verify: URL confirms catch-all redirect
```

### NAV-003: Protected Route Redirect -- Home

```
Test ID: NAV-003
Description: Accessing /home without auth redirects to /onboarding
Preconditions: Not authenticated (isAuthenticated: false)
Steps:
  1. Ensure isAuthenticated is false (may need to revert auth bypass)
  2. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/home"
  3. Wait 500ms
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding" (ProtectedRoute redirects)
Note: In browser (non-Tauri), the checkAuthStatus catch block preserves current state.
  If isAuthenticated was set to true earlier, it will remain true. Fresh page load may be needed.
Verify: URL confirms redirect
```

### NAV-004: Protected Route Redirect -- Agents

```
Test ID: NAV-004
Description: Accessing /agents without auth redirects to /onboarding
Preconditions: Not authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/agents"
  2. Wait 500ms
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding"
Verify: URL confirms redirect
```

### NAV-005: Protected Route Redirect -- Settings

```
Test ID: NAV-005
Description: Accessing /settings without auth redirects to /onboarding
Preconditions: Not authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/settings"
  2. Wait 500ms
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/onboarding"
Verify: URL confirms redirect
```

### NAV-006: Bottom Nav -- Home Tab

```
Test ID: NAV-006
Description: Clicking Home in bottom nav navigates to /home
Preconditions: Authenticated, on any page with bottom nav
Steps:
  1. Ensure authenticated, navigate to /agents
  2. Click "Home" in bottom nav
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/home"
Verify: URL confirms navigation
```

### NAV-007: Bottom Nav -- Agents Tab

```
Test ID: NAV-007
Description: Clicking Agents in bottom nav navigates to /agents
Preconditions: Authenticated, on /home
Steps:
  1. Navigate to /home
  2. Click "Agents" in bottom nav
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/agents"
Verify: URL confirms navigation
```

### NAV-008: Bottom Nav -- FAB Button

```
Test ID: NAV-008
Description: Clicking the FAB (+) button in bottom nav navigates to /agents
Preconditions: Authenticated, on /home
Steps:
  1. Navigate to /home
  2. Find the black circular FAB button (plus icon) in the center of bottom nav
  3. Click the FAB button
  4. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/agents"
Verify: URL confirms navigation
```

### NAV-009: Bottom Nav -- Settings Tab

```
Test ID: NAV-009
Description: Clicking Settings in bottom nav navigates to /settings
Preconditions: Authenticated, on /home
Steps:
  1. Navigate to /home
  2. Click "Settings" in bottom nav
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/settings"
Verify: URL confirms navigation
```

### NAV-010: Bottom Nav -- Stats Tab

```
Test ID: NAV-010
Description: Clicking Stats in bottom nav navigates to /home (placeholder)
Preconditions: Authenticated, on /agents
Steps:
  1. Navigate to /agents
  2. Click "Stats" in bottom nav
  3. Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
Expected Result:
  - URL pathname is "/home" (Stats routes to /home as a placeholder)
Verify: URL confirms navigation
```

### NAV-011: Setup Flow is Not Protected

```
Test ID: NAV-011
Description: Onboarding and setup routes are accessible without auth
Preconditions: Not authenticated
Steps:
  1. Call mcp__claude-in-chrome__navigate with url "http://localhost:1420/onboarding"
  2. Verify no redirect: window.location.pathname === "/onboarding"
  3. Navigate to "http://localhost:1420/setup/install"
  4. Verify: pathname === "/setup/install"
  5. Navigate to "http://localhost:1420/setup/connect"
  6. Verify: pathname === "/setup/connect"
  7. Navigate to "http://localhost:1420/setup/verify"
  8. Verify: pathname === "/setup/verify"
Expected Result:
  - All 4 setup routes are accessible without authentication
Verify: No redirects occur on any setup route
```

---

## Suite 12: Visual Regression

**Prerequisites**: Authenticated for protected routes

### VIS-001: Viewport Width Check

```
Test ID: VIS-001
Description: Verify the app renders correctly at 390px width
Preconditions: Browser resized to 390x844
Steps:
  1. Call mcp__claude-in-chrome__resize_window with width 390, height 844
  2. Navigate to /home (authenticated)
  3. Call mcp__claude-in-chrome__get_screenshot
  4. Call mcp__claude-in-chrome__javascript_tool with code:
     `JSON.stringify({innerWidth: window.innerWidth, innerHeight: window.innerHeight})`
Expected Result:
  - Window inner width is 390px (or close, accounting for scrollbar)
  - No horizontal overflow (no horizontal scrollbar)
  - Content fills the width appropriately
Verify: Screenshot shows properly contained layout
```

### VIS-002: No Horizontal Overflow

```
Test ID: VIS-002
Description: Verify no page has horizontal overflow at 390px
Preconditions: Authenticated, window at 390x844
Steps:
  For each route [/home, /agents, /settings, /add-funds, /agents/agent-research-001, /transactions/tx-001]:
  1. Navigate to the route
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.documentElement.scrollWidth > document.documentElement.clientWidth ? 'OVERFLOW' : 'OK'`
Expected Result:
  - All pages return "OK" (no horizontal overflow)
Verify: JavaScript check on each page
```

### VIS-003: Onboarding Screenshot

```
Test ID: VIS-003
Description: Capture baseline screenshot of onboarding
Steps:
  1. Navigate to /onboarding
  2. Call mcp__claude-in-chrome__get_screenshot
  3. Save/note the screenshot for future comparison
Expected Result:
  - Clean layout: centered content, logo, title, dots, button
  - No visual glitches, overlaps, or cut-off text
Verify: Visual inspection of screenshot
```

### VIS-004: Home Dashboard Screenshot

```
Test ID: VIS-004
Description: Capture baseline screenshot of home dashboard
Steps:
  1. Navigate to /home (authenticated)
  2. Call mcp__claude-in-chrome__get_screenshot
Expected Result:
  - Balance card is fully visible and not clipped
  - Action buttons side by side, equal width
  - Segment control properly centered
  - Activity items evenly spaced
  - Bottom nav flush with bottom edge
Verify: Visual inspection
```

### VIS-005: Agent Detail Screenshot

```
Test ID: VIS-005
Description: Capture baseline screenshot of agent detail (full scroll)
Steps:
  1. Navigate to /agents/agent-research-001 (authenticated)
  2. Call mcp__claude-in-chrome__get_screenshot (top of page)
  3. Scroll down to bottom
  4. Call mcp__claude-in-chrome__get_screenshot (bottom of page)
Expected Result:
  - Top: header, agent info, daily spend card
  - Bottom: spending controls, history, save button
  - No overlapping elements
Verify: Visual inspection of both screenshots
```

### VIS-006: Bottom Nav Elevation

```
Test ID: VIS-006
Description: Verify FAB button is properly elevated above the bottom nav bar
Steps:
  1. Navigate to /home (authenticated)
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `(() => {
       const fab = document.querySelector('.rounded-full.bg-black.text-white.-mt-\\[28px\\]');
       const nav = document.querySelector('nav');
       if (!fab || !nav) return 'elements not found';
       const fabRect = fab.getBoundingClientRect();
       const navRect = nav.getBoundingClientRect();
       return JSON.stringify({
         fabTop: fabRect.top,
         navTop: navRect.top,
         fabAboveNav: fabRect.top < navRect.top,
         fabOverlap: navRect.top - fabRect.top
       });
     })()`
Expected Result:
  - FAB top is above nav top (fabAboveNav: true)
  - FAB overlaps the nav bar by about 28px (the -mt-[28px] offset)
Verify: JavaScript confirms FAB positioning
```

### VIS-007: Text Truncation on Address

```
Test ID: VIS-007
Description: Wallet address is truncated in the balance card, full in Add Funds
Steps:
  1. Navigate to /home (authenticated)
  2. Call mcp__claude-in-chrome__get_page_text
  3. Check: address shows as "0x72AE...504B4" (truncated)
  4. Navigate to /add-funds
  5. Call mcp__claude-in-chrome__get_page_text
  6. Check: address shows full "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4"
Expected Result:
  - Home: truncated address
  - Add Funds: full address (with CSS truncation class, may still show full)
Verify: Text comparison between pages
```

---

## Suite 13: Known Issues Verification

Verify the status of each known GitHub issue.

### KI-001: OTP Input Boxes Not Visible (Issue #1)

```
Test ID: KI-001
Description: Verify whether OTP input boxes are visible or invisible
Preconditions: Navigate to /setup/verify
Steps:
  1. Navigate to /setup/verify
  2. Run OTP-002 steps (check input dimensions)
Result Options:
  - If inputs have 0 dimensions -> Issue #1 is STILL PRESENT
  - If inputs are visible with proper dimensions -> Issue #1 is FIXED
Record: Issue #1 status
```

### KI-002: install_skill Command Fake (Issue #2)

```
Test ID: KI-002
Description: Verify install_skill is faked with setState, not a real Tauri command
Preconditions: None
Steps:
  1. Read source: /Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/InstallSkill.tsx
  2. Verify: "Confirm Installation" button calls setState('success') directly, no tauriApi call
Expected Result:
  - No tauriApi.* call in the "Confirm Installation" handler
  - Transition is immediate (no async, no invoke)
Record: Issue #2 is STILL PRESENT (by design -- no backend command exists)
```

### KI-003: get_balance/get_address Stubs (Issue #3)

```
Test ID: KI-003
Description: Verify balance and address are from placeholder data, not backend
Preconditions: On /home (authenticated)
Steps:
  1. Read source: /Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Home.tsx
  2. Verify: Component imports from placeholder_data.json for wallet data
  3. Check: `const wallet = placeholderData.wallet` is used directly
Expected Result:
  - Home page uses placeholder data directly, not store data from Tauri
Record: Issue #3 is STILL PRESENT
```

### KI-004: export_wallet_history Not Implemented (Issue #4)

```
Test ID: KI-004
Description: Verify Export Wallet History button has no functional handler
Preconditions: On /settings
Steps:
  1. Run SET-007 steps
Expected Result:
  - Button exists but clicking does nothing
Record: Issue #4 is STILL PRESENT
```

### KI-005: buy_with_card Not Implemented (Issue #5)

```
Test ID: KI-005
Description: Verify Buy with Card button is disabled
Preconditions: On /add-funds
Steps:
  1. Run AFD-003 steps
Expected Result:
  - Button disabled, shows "Coming Soon"
Record: Issue #5 is STILL PRESENT
```

### KI-006: QR Code Placeholder (Issue #6)

```
Test ID: KI-006
Description: Verify QR code is a placeholder, not a real QR
Preconditions: On /add-funds
Steps:
  1. Navigate to /add-funds
  2. Call mcp__claude-in-chrome__javascript_tool with code:
     `document.querySelector('.border-dashed')?.innerHTML.includes('svg') ? 'icon placeholder' : 'real QR'`
Expected Result:
  - Returns "icon placeholder" (a Grid3x3 icon, not a real QR code)
  - Dashed border indicates placeholder
Record: Issue #6 is STILL PRESENT
```

### KI-007: resume_agent Command Missing (Issue #7)

```
Test ID: KI-007
Description: Verify resume_agent is not available; toggle only changes local UI
Preconditions: None
Steps:
  1. Read source: /Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/AgentDetail.tsx
  2. Verify: Toggle calls setIsPaused (local state), no tauriApi call for resume
  3. Grep for "resume_agent" in src/lib/tauri.ts
Expected Result:
  - No resume_agent function in tauriApi
  - Toggle is purely local state
Record: Issue #7 is STILL PRESENT
```

---

## Gmail OTP Retrieval Flow

This section provides detailed instructions for retrieving a real OTP code from Gmail when testing the full authentication flow. This requires:
1. The Tauri dev server running (`cargo tauri dev`), not just Vite
2. Gmail MCP tools loaded and working
3. The auth email: `dennison@dennisonbertram.com`

### Step-by-Step OTP Retrieval

```
PREREQUISITE: Ensure Gmail tools are loaded
  Action: Call ToolSearch with query "gmail"
  Result: Gmail MCP tools are available

STEP 1: Note the current time
  Action: Call Bash with command: date -u '+%Y/%m/%d %H:%M:%S'
  Purpose: To filter Gmail search to only recent messages
  Save: current_time variable

STEP 2: Trigger the auth flow
  2a. Navigate to http://localhost:1420/setup/connect
  2b. Type "dennison@dennisonbertram.com" in the email field
  2c. Click "Send code"
  2d. Wait for the request to complete (2-3 seconds)
  2e. Confirm navigation to /setup/verify OR note any error

  If error: The Tauri backend is not running. ABORT OTP flow.
  If success: Proceed to Step 3.

STEP 3: Wait for email delivery
  Action: Wait 10 seconds (Bash: sleep 10)
  Purpose: Allow time for Coinbase to send the OTP email

STEP 4: Search Gmail for the OTP email
  Action: Call mcp__gmail__searchMessages with:
    query: "from:noreply@coinbase.com subject:verification newer_than:1m"
    maxResults: 3
  Expected: At least 1 message returned
  Decision:
    - If no messages -> wait 10 more seconds and retry (up to 3 times)
    - If message found -> note the message ID, proceed to Step 5

STEP 5: Read the OTP email
  Action: Call mcp__gmail__getMessage with:
    messageId: <the ID from Step 4>
    format: "full"
  Expected: Email body contains a 6-digit verification code
  Extract: The 6-digit code (look for a pattern like \d{6} in the body)

  Extraction strategy:
    - Look for the code in the email body/snippet
    - The code is typically a standalone 6-digit number
    - It may be in a format like "Your verification code is: 123456"
    - Use regex to find the first \d{6} match

  Save: otp_code variable

STEP 6: Enter the OTP code
  6a. Ensure you are on /setup/verify
  6b. Click on the first OTP input box
  6c. Type each digit of the OTP code one at a time:
      - Type digit 1 (focus auto-advances)
      - Type digit 2
      - Type digit 3
      - Type digit 4
      - Type digit 5
      - Type digit 6
  6d. The onComplete handler should fire automatically after the 6th digit
  6e. Alternatively, click "Verify code" button

STEP 7: Verify authentication success
  Action: Wait 3 seconds, then check:
    - Call mcp__claude-in-chrome__javascript_tool with code: "window.location.pathname"
  Expected: URL is "/home" (successful auth redirects to home)
  Decision:
    - If /home -> AUTH SUCCESS, proceed with testing protected routes
    - If still on /setup/verify with error -> OTP may be wrong or expired
    - If error message shown -> capture screenshot and note error text

STEP 8: Take a success screenshot
  Action: Call mcp__claude-in-chrome__get_screenshot
  Purpose: Document successful authentication
```

### Alternative: Manual OTP Entry

If the Gmail MCP is unavailable, the human tester can:
1. Trigger the send code flow
2. Manually check their email for the OTP
3. Provide the 6-digit code to the agent
4. The agent enters the code using Chrome automation

---

## GIF Recording Instructions

Record a complete walkthrough GIF of the app for documentation purposes.

### Full App Walkthrough GIF

```
STEP 1: Load the GIF creator tool
  Action: Call ToolSearch with query "select:mcp__claude-in-chrome__gif_creator"

STEP 2: Start recording
  Action: Call mcp__claude-in-chrome__gif_creator with action "start"

STEP 3: Walk through the app (authenticated)
  Ensure auth bypass is active, then:

  3a. Navigate to /onboarding
      - Wait 2 seconds on each slide
      - Click "Next" through all 4 slides
      - Click "Get set up"

  3b. On Install Skill
      - Click "What changes?" to expand
      - Wait 1 second
      - Click "What changes?" to collapse
      - Click "Confirm Installation"
      - Wait 1 second on success screen
      - Click "Continue"

  3c. On Connect Coinbase
      - Type email "demo@example.com"
      - Wait 1 second
      - Navigate directly to /home (skip actual auth for recording)

  3d. On Home
      - Pause 2 seconds to show balance card
      - Scroll down slowly to show activity feed
      - Click "Agents" segment tab
      - Wait 1 second to show agent pills
      - Click "Overview" tab back

  3e. Navigate to /agents
      - Show all agents
      - Click "Active" segment
      - Click "All Agents" segment
      - Click Research Agent card

  3f. On Agent Detail
      - Scroll down slowly to show all sections
      - Click daily limit stepper plus button twice
      - Toggle the pause switch
      - Scroll to history section

  3g. Navigate back, then to /add-funds
      - Show QR placeholder and address
      - Click copy button

  3h. Navigate to /settings
      - Toggle a notification switch
      - Scroll to show full page

STEP 4: Stop recording
  Action: Call mcp__claude-in-chrome__gif_creator with action "stop"
  Expected: GIF file is saved/returned

STEP 5: Note the GIF file location for documentation
```

---

## Teardown Checklist

Execute these steps after all testing is complete.

### TD-001: Revert Auth Store (if modified)

```
If Method B was used for auth bypass:
  Action: Edit /Users/dennisonbertram/Develop/apps/agent-neo-bank/src/stores/authStore.ts
  Revert line 16 from `isAuthenticated: true` back to `isAuthenticated: false`
  Verify: File matches original content
```

### TD-002: Verify No Source Changes

```
Action: Run Bash command:
  cd /Users/dennisonbertram/Develop/apps/agent-neo-bank && git diff src/stores/authStore.ts
Expected: No diff (file is unchanged) OR only the expected auth bypass revert
```

### TD-003: Stop Dev Server (if started by agent)

```
If PF-004 was executed:
  Action: Run Bash command:
    lsof -ti :1420 | xargs kill 2>/dev/null; echo "cleaned up"
  Note: Only do this if the agent started the dev server. If it was already running, leave it.
```

### TD-004: Close Test Tab

```
Action: Close the test browser tab if desired
Note: This is optional; the tab can remain for manual inspection
```

### TD-005: Generate Test Report

```
Action: Copy the Test Results Template (below) and fill in all results
Output: Save to docs/testing/e2e-test-results-YYYY-MM-DD.md
```

---

## Test Results Template

Copy this table and fill in results after running all tests. Save to `docs/testing/e2e-test-results-YYYY-MM-DD.md`.

```markdown
# E2E Test Results -- [DATE]

## Environment
- **Tester**: [Agent ID or human name]
- **Browser**: Chrome [version]
- **Viewport**: 390x844px
- **Dev Server Port**: [1420 or other]
- **Tauri Backend**: [Running / Not Running]
- **Gmail Access**: [Available / Not Available]
- **Auth Method**: [Method A (JS injection) / Method B (source edit) / Method C / Real OTP]

## Summary
- **Total Tests**: 85
- **Passed**: [count]
- **Failed**: [count]
- **Skipped**: [count]
- **Blocked**: [count]

## Results

### Suite 1: Onboarding Flow (7 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| ONB-001 | First slide content | | |
| ONB-002 | Navigate to second slide | | |
| ONB-003 | Navigate to third slide | | |
| ONB-004 | Navigate to fourth slide | | |
| ONB-005 | Get set up navigates to install | | |
| ONB-006 | Indicator dot animation | | |
| ONB-007 | Slide transition animation | | |

### Suite 2: Install Skill (6 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| ISK-001 | Install screen initial state | | |
| ISK-002 | Expand what changes section | | |
| ISK-003 | Collapse what changes section | | |
| ISK-004 | Confirm installation success | | |
| ISK-005 | Continue navigates to connect | | |
| ISK-006 | Cancel button goes back | | |

### Suite 3: Auth Flow -- Connect Coinbase (7 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| CON-001 | Connect screen initial state | | |
| CON-002 | Send code button disabled when empty | | |
| CON-003 | Type email enables button | | |
| CON-004 | Send code triggers auth flow | | |
| CON-005 | Back button navigation | | |
| CON-006 | Enter key submits form | | |
| CON-007 | Error state display | | |

### Suite 4: Auth Flow -- Verify OTP (8 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| OTP-001 | Verify screen initial state | | |
| OTP-002 | OTP input boxes visibility | | |
| OTP-003 | Type OTP digits | | |
| OTP-004 | Backspace navigation | | |
| OTP-005 | Countdown timer | | |
| OTP-006 | Resend code disabled | | |
| OTP-007 | Verify button disabled | | |
| OTP-008 | Full OTP flow with Gmail | | |

### Suite 5: Home Dashboard (9 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| HOM-001 | Home screen layout | | |
| HOM-002 | Balance card data | | |
| HOM-003 | Add funds button | | |
| HOM-004 | Agents button | | |
| HOM-005 | Overview segment | | |
| HOM-006 | Agents segment | | |
| HOM-007 | Transaction item click | | |
| HOM-008 | Bottom nav bar | | |
| HOM-009 | Scroll behavior | | |

### Suite 6: Add Funds (4 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| AFD-001 | Add funds layout | | |
| AFD-002 | Copy wallet address | | |
| AFD-003 | Buy with card disabled | | |
| AFD-004 | Close returns to home | | |

### Suite 7: Agents List (7 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| AGT-001 | Agents list layout | | |
| AGT-002 | Active filter | | |
| AGT-003 | Archived filter | | |
| AGT-004 | All agents reset | | |
| AGT-005 | Agent card navigation | | |
| AGT-006 | Settings icon navigation | | |
| AGT-007 | Progress bar accuracy | | |

### Suite 8: Agent Detail (10 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| ADT-001 | Agent detail layout | | |
| ADT-002 | Back button navigation | | |
| ADT-003 | Toggle pause/resume | | |
| ADT-004 | Daily limit increase | | |
| ADT-005 | Daily limit decrease | | |
| ADT-006 | Per transaction stepper | | |
| ADT-007 | Approval threshold toggle | | |
| ADT-008 | Agent history section | | |
| ADT-009 | Save changes button | | |
| ADT-010 | Treasury pending status | | |

### Suite 9: Transaction Detail (6 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| TXD-001 | Send transaction detail | | |
| TXD-002 | Receive transaction detail | | |
| TXD-003 | Transaction not found | | |
| TXD-004 | Back navigation | | |
| TXD-005 | View on explorer button | | |
| TXD-006 | Awaiting approval tx | | |

### Suite 10: Settings (8 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| SET-001 | Settings layout | | |
| SET-002 | Agent requests toggle | | |
| SET-003 | Transaction completed toggle | | |
| SET-004 | Home button navigation | | |
| SET-005 | Reset coinbase dialog | | |
| SET-006 | Reset coinbase confirmed | | |
| SET-007 | Export wallet history | | |
| SET-008 | Version display | | |

### Suite 11: Navigation & Routing (11 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| NAV-001 | Default route redirect | | |
| NAV-002 | Unknown route redirect | | |
| NAV-003 | Protected route -- home | | |
| NAV-004 | Protected route -- agents | | |
| NAV-005 | Protected route -- settings | | |
| NAV-006 | Bottom nav -- home | | |
| NAV-007 | Bottom nav -- agents | | |
| NAV-008 | Bottom nav -- FAB | | |
| NAV-009 | Bottom nav -- settings | | |
| NAV-010 | Bottom nav -- stats | | |
| NAV-011 | Setup flow not protected | | |

### Suite 12: Visual Regression (7 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| VIS-001 | Viewport width check | | |
| VIS-002 | No horizontal overflow | | |
| VIS-003 | Onboarding screenshot | | |
| VIS-004 | Home dashboard screenshot | | |
| VIS-005 | Agent detail screenshot | | |
| VIS-006 | Bottom nav elevation | | |
| VIS-007 | Text truncation address | | |

### Suite 13: Known Issues Verification (7 tests)

| Test ID | Description | Result | Notes |
|---------|-------------|--------|-------|
| KI-001 | OTP input boxes (#1) | | |
| KI-002 | install_skill fake (#2) | | |
| KI-003 | get_balance stub (#3) | | |
| KI-004 | export_wallet_history (#4) | | |
| KI-005 | buy_with_card (#5) | | |
| KI-006 | QR placeholder (#6) | | |
| KI-007 | resume_agent missing (#7) | | |

## Screenshots

Link or embed screenshots captured during testing:
- [ ] ONB: Onboarding slides 1-4
- [ ] ISK: Install Skill (initial, expanded, success)
- [ ] CON: Connect Coinbase (empty, with email, error)
- [ ] OTP: Verify OTP (initial, with digits, error)
- [ ] HOM: Home (overview, agents segment)
- [ ] AFD: Add Funds (full page, copied state)
- [ ] AGT: Agents List (all, active, archived)
- [ ] ADT: Agent Detail (top, scrolled, paused)
- [ ] TXD: Transaction Detail (send, receive, not found)
- [ ] SET: Settings (full page, toggled)

## Known Issues Status

| Issue # | Title | Status |
|---------|-------|--------|
| #1 | OTP input boxes not visible | |
| #2 | install_skill faked | |
| #3 | get_balance/get_address stubs | |
| #4 | export_wallet_history not implemented | |
| #5 | buy_with_card not implemented | |
| #6 | QR code placeholder | |
| #7 | resume_agent missing | |
```

---

## Appendix: Quick Reference

### Route Map

| Route | Component | Protected | Bottom Nav Tab |
|-------|-----------|-----------|----------------|
| `/` | Redirect to /onboarding | No | -- |
| `/onboarding` | Onboarding.tsx | No | -- |
| `/setup/install` | InstallSkill.tsx | No | -- |
| `/setup/connect` | ConnectCoinbase.tsx | No | -- |
| `/setup/verify` | VerifyOtp.tsx | No | -- |
| `/home` | Home.tsx | Yes | Home |
| `/add-funds` | AddFunds.tsx | Yes | -- |
| `/agents` | AgentsList.tsx | Yes | Agents |
| `/agents/:agentId` | AgentDetail.tsx | Yes | -- |
| `/transactions/:txId` | TransactionDetail.tsx | Yes | -- |
| `/settings` | Settings.tsx | Yes | Settings |

### Placeholder Data Quick Reference

| Data Point | Value |
|------------|-------|
| Total Balance | $20.32 |
| USDC Balance | 20.00 |
| ETH Balance | 0.10 |
| Wallet Address | 0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4 |
| User Name | Dennison Bertram |
| User Email | dennison@dennisonbertram.com |
| User Initials | DB |
| App Version | 0.1.0 |
| Network | Base |
| Agent Count | 3 (Research Agent, Deploy Bot, Treasury) |
| Transaction Count | 5 (tx-001 through tx-005) |

### Agent Data

| ID | Name | Status | Accent Color | Daily Spent | Daily Cap |
|----|------|--------|-------------|-------------|-----------|
| agent-research-001 | Research Agent | active | #8FB5AA | $3.50 | $25.00 |
| agent-deploy-002 | Deploy Bot | active | #F2D48C | $0.0034 | $0.05 |
| agent-treasury-003 | Treasury | pending | #D9A58B | $0.00 | $50.00 |

### Transaction Data

| ID | Agent | Type | Amount | Asset | Status | Category |
|----|-------|------|--------|-------|--------|----------|
| tx-001 | Research Agent | send | -2.50 | USDC | confirmed | API Fee |
| tx-002 | Deploy Bot | send | -0.0034 | ETH | confirmed | Gas |
| tx-003 | Research Agent | send | -1.00 | USDC | confirmed | API Fee |
| tx-004 | Treasury | send | -5.00 | USDC | awaiting_approval | Swap |
| tx-005 | (none) | receive | +10.00 | USDC | confirmed | Deposit |
