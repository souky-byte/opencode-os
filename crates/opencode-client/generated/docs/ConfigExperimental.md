# ConfigExperimental

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**hook** | Option<[**models::ConfigExperimentalHook**](Config_experimental_hook.md)> |  | [optional]
**chat_max_retries** | Option<**f64**> | Number of retries for chat completions on failure | [optional]
**disable_paste_summary** | Option<**bool**> |  | [optional]
**batch_tool** | Option<**bool**> | Enable the batch tool | [optional]
**open_telemetry** | Option<**bool**> | Enable OpenTelemetry spans for AI SDK calls (using the 'experimental_telemetry' flag) | [optional]
**primary_tools** | Option<**Vec<String>**> | Tools that should only be available to primary agents. | [optional]
**continue_loop_on_deny** | Option<**bool**> | Continue the agent loop when a tool call is denied | [optional]
**mcp_timeout** | Option<**i32**> | Timeout in milliseconds for model context protocol (MCP) requests | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


