# ProviderList200ResponseAllInnerModelsValue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**name** | **String** |  | 
**family** | Option<**String**> |  | [optional]
**release_date** | **String** |  | 
**attachment** | **bool** |  | 
**reasoning** | **bool** |  | 
**temperature** | **bool** |  | 
**tool_call** | **bool** |  | 
**interleaved** | Option<[**models::ProviderList200ResponseAllInnerModelsValueInterleaved**](provider_list_200_response_all_inner_models_value_interleaved.md)> |  | [optional]
**cost** | Option<[**models::ProviderList200ResponseAllInnerModelsValueCost**](provider_list_200_response_all_inner_models_value_cost.md)> |  | [optional]
**limit** | [**models::ProviderList200ResponseAllInnerModelsValueLimit**](provider_list_200_response_all_inner_models_value_limit.md) |  | 
**modalities** | Option<[**models::ProviderList200ResponseAllInnerModelsValueModalities**](provider_list_200_response_all_inner_models_value_modalities.md)> |  | [optional]
**experimental** | Option<**bool**> |  | [optional]
**status** | Option<**String**> |  | [optional]
**options** | [**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md) |  | 
**headers** | Option<**std::collections::HashMap<String, String>**> |  | [optional]
**provider** | Option<[**models::ProviderList200ResponseAllInnerModelsValueProvider**](provider_list_200_response_all_inner_models_value_provider.md)> |  | [optional]
**variants** | Option<[**std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>**](std::collections::HashMap.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


