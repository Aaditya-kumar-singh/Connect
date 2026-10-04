-module(ybm_redis_pubsub).
-behaviour(gen_server).

-export([start_link/0, publish_status/1]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

-define(EVENTS, <<"ybm:reliability:events">>).
-define(STATUS, <<"ybm:reliability:status">>).

-record(state, {pub = undefined, sub = undefined}).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

publish_status(Payload) ->
    gen_server:cast(?MODULE, {publish, ?STATUS, Payload}).

init([]) ->
    Host = application:get_env(ybm_reliability, redis_host, "127.0.0.1"),
    Port = application:get_env(ybm_reliability, redis_port, 6379),
    Password = application:get_env(ybm_reliability, redis_password, ""),
    process_flag(trap_exit, true),
    self() ! connect,
    {ok, #state{pub = {Host, Port, Password}}}.

handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast({publish, Channel, Payload}, State = #state{pub = {Host, Port, Password}}) ->
    case eredis:start_link(Host, Port, 0, Password, no_reconnect, 5000) of
        {ok, Client} ->
            Result = eredis:q(Client, ["PUBLISH", binary_to_list(Channel), Payload]),
            eredis_client:stop(Client),
            _ = Result,
            {noreply, State};
        {error, Reason} ->
            error_logger:error_msg("reliability Redis publish failed: ~p", [Reason]),
            {noreply, State}
    end;
handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(connect, State = #state{pub = {Host, Port, Password}}) ->
    case eredis_sub:start_link(Host, Port, Password, 1000, infinity, drop) of
        {ok, Sub} ->
            eredis_sub:controlling_process(Sub),
            eredis_sub:subscribe(Sub, [?EVENTS]),
            {noreply, State#state{sub = Sub}};
        {error, Reason} ->
            error_logger:error_msg("reliability Redis subscribe failed: ~p", [Reason]),
            erlang:send_after(2000, self(), connect),
            {noreply, State}
    end;
handle_info({message, _Channel, Payload, Sub}, State = #state{sub = Sub}) ->
    eredis_sub:ack_message(Sub),
    case Payload of
        <<"{" , _/binary>> ->
            case binary:match(Payload, <<"\"event\":\"shutdown\"">>) of
                {_, _} -> application:stop(ybm_reliability);
                nomatch -> ok
            end;
        _ -> ok
    end,
    {noreply, State};
handle_info({eredis_disconnected, _Sub}, State) ->
    erlang:send_after(2000, self(), connect),
    {noreply, State};
handle_info({'EXIT', _Pid, _Reason}, State) ->
    self() ! connect,
    {noreply, State};
handle_info(_Info, State) ->
    {noreply, State}.

terminate(_Reason, #state{sub = Sub}) ->
    case Sub of
        undefined -> ok;
        _ -> catch eredis_sub:stop(Sub)
    end,
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.
