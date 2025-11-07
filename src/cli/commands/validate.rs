//! Validate command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use tabled::{Table, Tabled};
use tracing::info;

use crate::cli::ValidateArgs;
use crate::config::Config;
use llm_latency_lens_core::TimingEngine;
use llm_latency_lens_providers::{create_provider, MessageRole, Provider, StreamingRequest};

/// Run the validate command
pub async fn run(
    args: ValidateArgs,
    mut config: Config,
    json_output: bool,
    quiet: bool,
) -> Result<()> {
    info!("Starting validate command");

    // Merge CLI overrides
    if let Some(ref provider) = args.provider {
        config.merge_cli_overrides(provider, args.api_key.clone(), args.endpoint.clone());
    }

    // Determine which providers to validate
    let providers_to_validate: Vec<String> = if let Some(provider) = args.provider {
        vec![provider]
    } else {
        config.providers.keys().cloned().collect()
    };

    if providers_to_validate.is_empty() {
        anyhow::bail!("No providers configured. Please add provider configuration.");
    }

    if !quiet {
        println!(
            "{} Validating {} provider(s)...",
            "=>".bright_cyan().bold(),
            providers_to_validate.len().to_string().bright_white().bold()
        );
        println!();
    }

    // Results for each provider
    #[derive(Debug)]
    struct ValidationResult {
        provider: String,
        config_valid: bool,
        api_key_present: bool,
        connectivity: bool,
        test_request: Option<bool>,
        error: Option<String>,
    }

    let mut results: Vec<ValidationResult> = Vec::new();

    // Validate each provider
    for provider_name in providers_to_validate {
        if !quiet {
            println!(
                "{} Validating {}...",
                "=>".bright_cyan(),
                provider_name.bright_yellow()
            );
        }

        let mut result = ValidationResult {
            provider: provider_name.clone(),
            config_valid: false,
            api_key_present: false,
            connectivity: false,
            test_request: None,
            error: None,
        };

        // Check if provider is configured
        let provider_config = match config.get_provider(&provider_name) {
            Ok(cfg) => {
                result.config_valid = true;
                cfg
            }
            Err(e) => {
                result.error = Some(format!("Configuration error: {}", e));
                results.push(result);
                continue;
            }
        };

        // Check API key
        let api_key = match &provider_config.api_key {
            Some(key) => {
                result.api_key_present = true;
                key.clone()
            }
            None => {
                result.error = Some("API key not found".to_string());
                results.push(result);
                continue;
            }
        };

        // Create provider and test connectivity
        let provider = match create_provider(&provider_name, api_key) {
            Ok(p) => p,
            Err(e) => {
                result.error = Some(format!("Failed to create provider: {}", e));
                results.push(result);
                continue;
            }
        };

        // Test health check
        match provider.health_check().await {
            Ok(_) => {
                result.connectivity = true;
                if !quiet {
                    println!("  {} Configuration valid", "✓".bright_green());
                    println!("  {} API key present", "✓".bright_green());
                    println!("  {} Connectivity OK", "✓".bright_green());
                }
            }
            Err(e) => {
                result.error = Some(format!("Health check failed: {}", e));
                if !quiet {
                    println!("  {} Configuration valid", "✓".bright_green());
                    println!("  {} API key present", "✓".bright_green());
                    println!("  {} Connectivity failed: {}", "✗".bright_red(), e);
                }
                results.push(result);
                continue;
            }
        }

        // Test with a simple request if requested
        if args.test_request {
            if !quiet {
                println!("  {} Running test request...", "→".bright_cyan());
            }

            let test_result = run_test_request(&*provider).await;

            match test_result {
                Ok(_) => {
                    result.test_request = Some(true);
                    if !quiet {
                        println!("  {} Test request successful", "✓".bright_green());
                    }
                }
                Err(e) => {
                    result.test_request = Some(false);
                    result.error = Some(format!("Test request failed: {}", e));
                    if !quiet {
                        println!("  {} Test request failed: {}", "✗".bright_red(), e);
                    }
                }
            }
        }

        if !quiet {
            println!();
        }

        results.push(result);
    }

    // Output summary
    if json_output {
        let json_data: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "provider": r.provider,
                    "valid": r.config_valid && r.api_key_present && r.connectivity && r.test_request.unwrap_or(true),
                    "config_valid": r.config_valid,
                    "api_key_present": r.api_key_present,
                    "connectivity": r.connectivity,
                    "test_request": r.test_request,
                    "error": r.error,
                })
            })
            .collect();

        let output = if quiet {
            serde_json::to_string(&json_data)?
        } else {
            serde_json::to_string_pretty(&json_data)?
        };

        println!("{}", output);
    } else if !quiet {
        println!("{}", "Validation Summary".bright_cyan().bold().underline());
        println!();

        #[derive(Tabled)]
        struct SummaryRow {
            #[tabled(rename = "Provider")]
            provider: String,
            #[tabled(rename = "Config")]
            config: String,
            #[tabled(rename = "API Key")]
            api_key: String,
            #[tabled(rename = "Connectivity")]
            connectivity: String,
            #[tabled(rename = "Test Request")]
            test_request: String,
            #[tabled(rename = "Status")]
            status: String,
        }

        let summary_rows: Vec<_> = results
            .iter()
            .map(|r| {
                let status = if r.config_valid
                    && r.api_key_present
                    && r.connectivity
                    && r.test_request.unwrap_or(true)
                {
                    "✓ Valid".bright_green().to_string()
                } else {
                    format!("✗ {}", r.error.as_ref().unwrap_or(&"Failed".to_string()))
                        .bright_red()
                        .to_string()
                };

                SummaryRow {
                    provider: r.provider.clone(),
                    config: if r.config_valid {
                        "✓".bright_green().to_string()
                    } else {
                        "✗".bright_red().to_string()
                    },
                    api_key: if r.api_key_present {
                        "✓".bright_green().to_string()
                    } else {
                        "✗".bright_red().to_string()
                    },
                    connectivity: if r.connectivity {
                        "✓".bright_green().to_string()
                    } else {
                        "✗".bright_red().to_string()
                    },
                    test_request: match r.test_request {
                        Some(true) => "✓".bright_green().to_string(),
                        Some(false) => "✗".bright_red().to_string(),
                        None => "-".bright_black().to_string(),
                    },
                    status,
                }
            })
            .collect();

        println!("{}", Table::new(summary_rows));
        println!();

        // Overall summary
        let all_valid = results.iter().all(|r| {
            r.config_valid
                && r.api_key_present
                && r.connectivity
                && r.test_request.unwrap_or(true)
        });

        if all_valid {
            println!(
                "{} All providers validated successfully!",
                "✓".bright_green().bold()
            );
        } else {
            let failed_count = results
                .iter()
                .filter(|r| {
                    !r.config_valid
                        || !r.api_key_present
                        || !r.connectivity
                        || !r.test_request.unwrap_or(true)
                })
                .count();

            println!(
                "{} {} provider(s) failed validation",
                "✗".bright_red().bold(),
                failed_count
            );

            // Show errors
            for result in &results {
                if let Some(ref error) = result.error {
                    println!(
                        "  {} {}: {}",
                        "✗".bright_red(),
                        result.provider.bright_yellow(),
                        error
                    );
                }
            }
        }
    }

    Ok(())
}

/// Run a simple test request to validate the provider
async fn run_test_request(provider: &dyn Provider) -> Result<()> {
    let models = provider.supported_models();
    let model = models
        .first()
        .context("No supported models available")?
        .clone();

    let request = StreamingRequest::builder()
        .model(model)
        .message(MessageRole::User, "Hi")
        .max_tokens(5)
        .build();

    let timing_engine = TimingEngine::new();

    // Execute the request
    let _result = provider.complete(request, &timing_engine).await?;

    Ok(())
}
