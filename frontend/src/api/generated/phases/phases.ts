/**
 * Phases API hooks
 * Manual addition for task phases endpoint
 */
import { useQuery } from '@tanstack/react-query';
import type {
  DataTag,
  QueryFunction,
  QueryKey,
  UseQueryOptions,
  UseQueryResult
} from '@tanstack/react-query';

import type { PhasesResponse } from '../model';
import { customFetch } from '../../../lib/api-fetcher';

type SecondParameter<T extends (...args: never) => unknown> = Parameters<T>[1];

export type getTaskPhasesResponse200 = {
  data: PhasesResponse
  status: 200
}

export type getTaskPhasesResponse404 = {
  data: void
  status: 404
}

export type getTaskPhasesResponseSuccess = (getTaskPhasesResponse200) & {
  headers: Headers;
};
export type getTaskPhasesResponseError = (getTaskPhasesResponse404) & {
  headers: Headers;
};

export type getTaskPhasesResponse = (getTaskPhasesResponseSuccess | getTaskPhasesResponseError)

export const getGetTaskPhasesUrl = (id: string) => {
  return `/api/tasks/${id}/phases`
}

export const getTaskPhases = async (id: string, options?: RequestInit): Promise<getTaskPhasesResponse> => {
  return customFetch<getTaskPhasesResponse>(getGetTaskPhasesUrl(id), {
    ...options,
    method: 'GET'
  });
}

export const getGetTaskPhasesQueryKey = (id?: string) => {
  return [`/api/tasks/${id}/phases`] as const;
}

export const getGetTaskPhasesQueryOptions = <TData = Awaited<ReturnType<typeof getTaskPhases>>, TError = void>(
  id: string,
  options?: { query?: Partial<UseQueryOptions<Awaited<ReturnType<typeof getTaskPhases>>, TError, TData>>, request?: SecondParameter<typeof customFetch> }
) => {
  const { query: queryOptions, request: requestOptions } = options ?? {};
  const queryKey = queryOptions?.queryKey ?? getGetTaskPhasesQueryKey(id);

  const queryFn: QueryFunction<Awaited<ReturnType<typeof getTaskPhases>>> = ({ signal }) =>
    getTaskPhases(id, { signal, ...requestOptions });

  return {
    queryKey,
    queryFn,
    enabled: !!(id),
    ...queryOptions
  } as UseQueryOptions<Awaited<ReturnType<typeof getTaskPhases>>, TError, TData> & {
    queryKey: DataTag<QueryKey, TData, TError>
  }
}

export type GetTaskPhasesQueryResult = NonNullable<Awaited<ReturnType<typeof getTaskPhases>>>
export type GetTaskPhasesQueryError = void

export function useGetTaskPhases<TData = Awaited<ReturnType<typeof getTaskPhases>>, TError = void>(
  id: string,
  options?: { query?: Partial<UseQueryOptions<Awaited<ReturnType<typeof getTaskPhases>>, TError, TData>>, request?: SecondParameter<typeof customFetch> }
): UseQueryResult<TData, TError> & { queryKey: DataTag<QueryKey, TData, TError> } {
  const queryOptions = getGetTaskPhasesQueryOptions(id, options)
  const query = useQuery(queryOptions) as UseQueryResult<TData, TError> & { queryKey: DataTag<QueryKey, TData, TError> };
  query.queryKey = queryOptions.queryKey;
  return query;
}
