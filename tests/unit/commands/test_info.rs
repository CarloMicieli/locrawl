#[cfg(test)]
mod tests {
    use locrawl::commands::info;

    #[tokio::test]
    async fn test_info_command_runs_without_error() {
        // This test will fail until info::run is implemented
        // It ensures the command can be called without panicking
        let result = info::run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_info_output_format() {
        // TODO: Test that output contains expected elements
        // This will require capturing stdout or refactoring to return output
    }
}