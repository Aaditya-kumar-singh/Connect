-module(ybm_health_aggregator).
-behaviour(gen_server).

-export([start_link/0, health/0, health_json/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

health() ->
    gen_server:call(?MODULE, health).

init([]) ->
    Port = application:get_env(ybm_reliability, health_port, 8081),
    inets:start(),
    {ok, HttpdPid} = inets:start(httpd, [
        {port, Port},
        {server_name, "ybm-reliability"},
        {server_root, "/tmp/ybm-reliability"},
        {document_root, "/tmp/ybm-reliability"},
        {bind_address, "0.0.0.0"},
        {modules, [ybm_health_httpd, mod_alias, mod_get, mod_head, mod_log]}
    ]),
    RestartCount = persistent_term:get({?MODULE, restart_count}, 0) + 1,
    persistent_term:put({?MODULE, restart_count}, RestartCount),
    {ok, #{httpd_pid => HttpdPid, restart_count => RestartCount}}.

handle_call(health, _From, State) ->
    {reply, build_health(), State};
handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(_Info, State) ->
    {noreply, State}.

build_health() ->
    States = [
        {postgres, ybm_circuit_breaker:get_state(postgres)},
        {redis, ybm_circuit_breaker:get_state(redis)},
        {storage, ybm_circuit_breaker:get_state(storage)}
    ],
    Status = case lists:any(fun({_Name, S}) -> maps:get(status, S) =/= closed end, States) of
        true -> degraded;
        false -> healthy
    end,
    #{status => Status,
      restart_count => persistent_term:get({?MODULE, restart_count}, 0),
      dependencies => maps:from_list(States)}.

health_json() ->
    #{status := Status, restart_count := RestartCount, dependencies := Dependencies} = build_health(),
    Pg = maps:get(postgres, Dependencies),
    Redis = maps:get(redis, Dependencies),
    Storage = maps:get(storage, Dependencies),
    iolist_to_binary(io_lib:format(
        "{\"status\":\"~p\",\"restart_count\":~p,\"dependencies\":{\"postgres\":\"~p\",\"redis\":\"~p\",\"storage\":\"~p\"}}",
        [Status, RestartCount, maps:get(status, Pg), maps:get(status, Redis), maps:get(status, Storage)])).

terminate(_Reason, State) ->
    case maps:get(httpd_pid, State, undefined) of
        undefined -> ok;
        Pid -> catch inets:stop(httpd, Pid)
    end,
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.
