// Non-compliant Go code — missing all ANEL flags
export const sampleGoNonCompliant = `
package cmd

import (
    "fmt"
    "github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
    Use:   "search <query>",
    Short: "Search documents",
    RunE:  handleSearch,
}

func handleSearch(cmd *cobra.Command, args []string) error {
    if len(args) == 0 {
        return fmt.Errorf("query required")
    }
    results, err := doSearch(args[0])
    if err != nil {
        return fmt.Errorf("search failed: %w", err)
    }
    fmt.Println(results)
    return nil
}
`;

// Fully ANEL-compliant Go code
export const sampleGoCompliant = `
package cmd

import (
    "encoding/json"
    "fmt"
    "os"
    "github.com/spf13/cobra"
    "my-cli/anel"
)

var searchCmd = &cobra.Command{
    Use:   "search <query>",
    Short: "Search documents",
    RunE:  handleSearch,
}

func init() {
    searchCmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
    searchCmd.Flags().Bool("dry-run", false, "Validate without executing")
    searchCmd.Flags().String("output-format", "ndjson", "Output format")
}

func handleSearch(cmd *cobra.Command, args []string) error {
    traceID := os.Getenv("AGENT_TRACE_ID")
    identityToken := os.Getenv("AGENT_IDENTITY_TOKEN")
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    dryRun, _ := cmd.Flags().GetBool("dry-run")

    if emitSpec {
        spec := anel.GetSpec("search")
        json.NewEncoder(os.Stdout).Encode(spec)
        return nil
    }

    if dryRun {
        fmt.Fprintf(os.Stderr, \`{"dry_run": true, "command": "search", "trace_id": "%s"}\n\`, traceID)
        return nil
    }

    if len(args) == 0 {
        err := anel.NewError(anel.E_INVALID_INPUT, "query required").
            WithRecoveryHint("CHECK_ARGS", "Provide a query argument").
            WithTraceID(traceID)
        err.EmitStderr()
        return err
    }

    results, err := doSearch(args[0])
    if err != nil {
        return err
    }

    encoder := json.NewEncoder(os.Stdout)
    for _, r := range results {
        encoder.Encode(r)
    }
    return nil
}
`;

// Partially compliant — has some flags but missing error format
export const sampleGoPartial = `
package cmd

import (
    "encoding/json"
    "fmt"
    "os"
    "github.com/spf13/cobra"
)

var searchCmd = &cobra.Command{
    Use:   "search <query>",
    Short: "Search documents",
    RunE:  handleSearch,
}

func init() {
    searchCmd.Flags().Bool("emit-spec", false, "Output ANEL specification")
    searchCmd.Flags().Bool("dry-run", false, "Validate without executing")
}

func handleSearch(cmd *cobra.Command, args []string) error {
    emitSpec, _ := cmd.Flags().GetBool("emit-spec")
    dryRun, _ := cmd.Flags().GetBool("dry-run")

    if emitSpec {
        json.NewEncoder(os.Stdout).Encode(map[string]string{"command": "search"})
        return nil
    }

    if dryRun {
        fmt.Fprintf(os.Stderr, \`{"dry_run": true}\n\`)
        return nil
    }

    if len(args) == 0 {
        return fmt.Errorf("query required")
    }

    results, err := doSearch(args[0])
    if err != nil {
        return err
    }
    fmt.Println(results)
    return nil
}
`;
