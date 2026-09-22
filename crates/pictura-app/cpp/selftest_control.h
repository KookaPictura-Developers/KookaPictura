#pragma once

namespace pictura {
class PicturaMainWindow;

// Runs the mcp_control checks: starts a ControlServer on a temporary socket,
// round-trips status/list_commands/dispatch_command through a QLocalSocket, and
// asserts the bad_request/unknown_method error paths and the user-only socket.
// Check codes start at 463. Returns 0 when all pass, otherwise the failure code.
int runControlChecks(PicturaMainWindow& frame);
} // namespace pictura
