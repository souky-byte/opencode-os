# Model

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**provider_id** | **String** |  | 
**api** | [**models::ModelApi**](Model_api.md) |  | 
**name** | **String** |  | 
**family** | Option<**String**> |  | [optional]
**capabilities** | [**models::ModelCapabilities**](Model_capabilities.md) |  | 
**cost** | [**models::ModelCost**](Model_cost.md) |  | 
**limit** | [**models::ProviderList200ResponseAllInnerModelsValueLimit**](provider_list_200_response_all_inner_models_value_limit.md) |  | 
**status** | **String** |  | 
**options** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  | 
**headers** | **std::collections::HashMap<String, String>** |  | 
**release_date** | **String** |  | 
**variants** | Option<[**std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>**](std::collections::HashMap.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


