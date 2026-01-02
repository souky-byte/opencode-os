# AgentConfig

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**model** | Option<**String**> |  | [optional]
**temperature** | Option<**f64**> |  | [optional]
**top_p** | Option<**f64**> |  | [optional]
**prompt** | Option<**String**> |  | [optional]
**tools** | Option<**std::collections::HashMap<String, bool>**> |  | [optional]
**disable** | Option<**bool**> |  | [optional]
**description** | Option<**String**> | Description of when to use the agent | [optional]
**mode** | Option<**String**> |  | [optional]
**color** | Option<**String**> | Hex color code for the agent (e.g., #FF5733) | [optional]
**max_steps** | Option<**i32**> | Maximum number of agentic iterations before forcing text-only response | [optional]
**permission** | Option<[**models::AgentConfigPermission**](AgentConfig_permission.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


