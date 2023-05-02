import typing
import itertools
from typing import Iterator, Callable
from enum import Enum

Hand = Enum('Hand', ['LEFT', 'RIGHT'])

class Finger(Enum):
    LP = 0
    LR = 1
    LM = 2
    LI = 3
    LT = 4
    RT = 5
    RI = 6
    RM = 7
    RR = 8
    RP = 9

    def hand(self):
        if self <= LT:
            return Hand.LEFT
        else:
            return Hand.RIGHT

class Pos:
    def __init__(self, col, row):
        self.col = col
        self.row = row

class Keyboard:
    def x(p: Pos) -> float:
        pass
    def y(p: Pos) -> float:
        pass
    def finger(p: Pos) -> Finger:
        pass
    def positions() -> Iterator[Pos]:
        pass

class Matrix(Keyboard):
    def x(p: Pos) -> float:
        return p.col
    def y(p: Pos) -> float:
        return p.row
    def finger(p: Pos) -> Finger:
        if p.col == 3 or p.col == 4:
            return Finger.LI
        elif p.col == 5 or p.col == 6:
            return Finger.RI
        else
            return Finger(p.col)
    def positions() -> Iterator[Pos]:
        for col in range(0, 10):
            for row in range(0, 3):
                yield Pos(col, row)

class Metric:
    def __init__(self, nstroke_length: int, function: Callable[[Keyboard, list[Pos]], float], ignored_vals=[0]):
        self.function = function
        self.nstroke_length = nstroke_length
        self.ignored_vals = ignored_vals
    def eval(self, kb: Keyboard, p: list[Pos]) -> float:
        return self.function(kb, p)

def metric_nstrokes(kb: Keyboard, m: Metric):
    return filter(lambda x: x[1] not in m.ignored_vals,
                  map(lambda p: [p, m.eval(k, p)],
                      itertools.product(kb.positions(), repeat=m.nstroke_length)))

def sfb(kb: Keyboard, p: list[Pos]) -> float:
    return float(
        p[0].col != p[1].col and
        p[0].row != p[1].row and
        kb.finger(p[0]) == kb.finger(p[1]))

sfb = Metric(2, sfb)
