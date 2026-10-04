-module(ybm_recovery_coordinator).
-behaviour(gen_server).

-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    {ok, #{}}.

handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info({circuit_transition, Name, Old, New}, State) ->
    case {Old, New} of
        {open, closed} ->
            publish(recovery, Name, New);
        {half_open, closed} ->
            publish(recovery, Name, New);
        {closed, open} ->
            publish(degraded, Name, New);
        {half_open, open} ->
            publish(degraded, Name, New);
        _ ->
            ok
    end,
    {noreply, State};
handle_info(_Info, State) ->
    {noreply, State}.

publish(Event, Name, Status) ->
    Payload = io_lib:format(
        "{\"event\":\"~p\",\"dependency\":\"~p\",\"status\":\"~p\",\"source\":\"erlang\"}",
        [Event, Name, Status]),
    ybm_redis_pubsub:publish_status(iolist_to_binary(Payload)).

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.
