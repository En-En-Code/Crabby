#!/bin/sh

mem=$(free|awk '/^Mem:/{print $4}')
nthreads=$(nproc)
memper=$((mem*4/5/nthreads/1024/2))

echo $memper
if [ ! -e testing/results ];
then mkdir ./testing/results
fi
output=./testing/results/games.pgn
book=./testing/book.pgn

engine1=${3:-'./target/release/crabby'}
engine2=${2:-'./target/release/crabby'}
numgames=${1:-'2000'}

cutechess-cli \
-engine cmd=$engine1 proto=uci initstr="setoption name hash value $memper" \
-engine cmd=$engine2 proto=uci initstr="setoption name hash value $memper" \
-each tc=40/60 timemargin=0 \
-games $numgames -repeat -recover \
-concurrency $nthreads -openings file=$book -pgnout $output
