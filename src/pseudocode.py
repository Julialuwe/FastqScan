import argparse
from pathlib import Path

def parse_args():
    parser = argparse.ArgumentParser(description="FASTQ Quality Control")
    parser.add_argument("-1", "--read1", type=Path, required=True, help="Path to first FASTQ file (R1)")
    parser.add_argument("-2", "--read2", type=Path, help="Path to second FASTQ file (R2, optional)")
    return parser.parse_args()

#in rust with clap and PathBuf
import gzip

def open_fastq(path):
    if str(path).endswith(".gz"):
        return gzip.open(path, "rt")
    else:
        return open(path, "r")

import json
from abc import ABC, abstractmethod
from collections import defaultdict

class FastqRecord:
    def __init__(self, seq="", qual=""):
        self.seq = seq
        self.qual = qual

class Statistic(ABC):
    @abstractmethod
    def process(self, record: FastqRecord):
        pass

    @abstractmethod
    def report_json(self):
        pass


class GcContentPerPosition(Statistic):
    def __init__(self):
        self.gc_counts = []
        self.total_counts = []

    def process(self, record):
        for i, base in enumerate(record.seq):
            if i >= len(self.gc_counts):
                self.gc_counts.append(0)
                self.total_counts.append(0)
            if base in 'GC':
                self.gc_counts[i] += 1
            self.total_counts[i] += 1

    def report_json(self):
        result = []
        for gc, total in zip(self.gc_counts, self.total_counts):
            result.append(gc / total if total > 0 else 0.0)
        return {"average_gc_content_per_position": result}


class BaseCompositionStatistic(Statistic):
    def __init__(self):
        self.base_counts = []

    def process(self, record):
        for i, base in enumerate(record.seq):
            if i >= len(self.base_counts):
                self.base_counts.append(defaultdict(int))
            self.base_counts[i][base] += 1

    def report_json(self):
        output = []
        for pos in self.base_counts:
            total = sum(pos.values())
            if total == 0:
                output.append({b: 0.0 for b in "ACGTN"})
            else:
                output.append({b: pos.get(b, 0) / total for b in "ACGTN"})
        return {"base_composition_per_position": output}


class BaseQualityPosStatistic(Statistic):
    def __init__(self):
        self.total_qualities = []
        self.counts = []

    def process(self, record):
        for i, q in enumerate(record.qual):
            phred = ord(q) - 33
            if i >= len(self.total_qualities):
                self.total_qualities.append(0)
                self.counts.append(0)
            self.total_qualities[i] += phred
            self.counts[i] += 1

    def report_json(self):
        return {
            "average_base_quality_per_position": [
                total / count if count else 0.0
                for total, count in zip(self.total_qualities, self.counts)
            ]
        }


class ReadQualityStatistic(Statistic):
    def __init__(self):
        self.total_quality = 0
        self.read_count = 0

    def process(self, record):
        qualities = [ord(q) - 33 for q in record.qual]
        self.total_quality += sum(qualities) / len(qualities)
        self.read_count += 1

    def report_json(self):
        avg = self.total_quality / self.read_count if self.read_count else 0.0
        return {"average_read_quality": avg}


class WorkflowRunner:
    def __init__(self):
        self.statistics = []

    def register(self, stat):
        self.statistics.append(stat)

    def process(self, file_handle):
        while True:
            header = file_handle.readline()
            if not header:
                break
            seq = file_handle.readline().strip()
            file_handle.readline()  # skip +
            qual = file_handle.readline().strip()
            record = FastqRecord(seq, qual)
            for stat in self.statistics:
                stat.process(record)

    def finalize(self):
        output = {}
        for stat in self.statistics:
            output.update(stat.report_json())
        return output



def main():
    args = parse_args()

    try:
        reader1 = open_fastq(args.read1)
    except Exception as e:
        print(f"Error opening file 1: {e}", file=sys.stderr)
        sys.exit(1)

    runner = WorkflowRunner()
    runner.register(ReadQualityStatistic())
    runner.register(BaseQualityPosStatistic())
    runner.register(BaseCompositionStatistic())
    runner.register(GcContentPerPosition())

    runner.process(reader1)

    if args.read2:
        try:
            reader2 = open_fastq(args.read2)
        except Exception as e:
            print(f"Error opening file 2: {e}", file=sys.stderr)
            sys.exit(1)
        runner.process(reader2)

    results = runner.finalize()
    print(json.dumps(results, indent=2))
    print("Parsing done!")

if __name__ == "__main__":
    main()
