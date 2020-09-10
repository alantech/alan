Include build_tools.sh

Describe "Interfaces"
  Describe "basic matching"
    before() {
      sourceToAll "
        from @std/app import start, print, exit

        interface Stringifiable {
          toString(Stringifiable): string
        }

        fn quoteAndPrint(toQuote: Stringifiable) {
          print(\"'\" + toString(toQuote) + \"'\")
        }

        on start {
          quoteAndPrint('Hello, World')
          quoteAndPrint(5)
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "'Hello, World'
'5'"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "'Hello, World'
'5'"
    End
  End

  Describe "import behavior"
    before() {
      sourceToFile datetime.ln "
        from @std/app import print

        export type Year {
          year: int32
        }

        export type YearMonth {
          year: int32
          month: int8
        }

        export type Date {
          year: int32
          month: int8
          day: int8
        }

        export type Hour {
          hour: int8
        }

        export type HourMinute {
          hour: int8
          minute: int8
        }

        export type Time {
          hour: int8
          minute: int8
          second: float64
        }

        export type DateTime {
          date: Date
          time: Time
          timezone: HourMinute
        }

        export fn makeYear(year: int32): Year {
          return new Year {
            year = year
          }
        }

        export fn makeYear(year: int64): Year {
          return new Year {
            year = toInt32(year)
          }
        }

        export fn makeYearMonth(year: int32, month: int8): YearMonth {
          return new YearMonth {
            year = year
            month = month
          }
        }

        export fn makeYearMonth(y: Year, month: int64): YearMonth {
          return new YearMonth {
            year = y.year
            month = toInt8(month)
          }
        }

        export fn makeDate(year: int32, month: int8, day: int8): Date {
          return new Date {
            year = year
            month = month
            day = day
          }
        }

        export fn makeDate(ym: YearMonth, day: int64): Date {
          return new Date {
            year = ym.year
            month = ym.month
            day = toInt8(day)
          }
        }

        export fn makeHour(hour: int8): Hour {
          return new Hour {
            hour = hour
          }
        }

        export fn makeHourMinute(hour: int8, minute: int8): HourMinute {
          return new HourMinute {
            hour = hour
            minute = minute
          }
        }

        export fn makeHourMinute(hour: int64, minute: int64): HourMinute {
          return new HourMinute {
            hour = toInt8(hour)
            minute = toInt8(minute)
          }
        }

        export fn makeHourMinute(h: Hour, minute: int8): HourMinute {
          return new HourMinute {
            hour = h.hour
            minute = minute
          }
        }

        export fn makeTime(hour: int8, minute: int8, second: float64): Time {
          return new Time {
            hour = hour
            minute = minute
            second = second
          }
        }

        export fn makeTime(hm: HourMinute, second: float64): Time {
          return new Time {
            hour = hm.hour
            minute = hm.minute
            second = second
          }
        }

        export fn makeTime(hm: HourMinute, second: int64): Time {
          return new Time {
            hour = hm.hour
            minute = hm.minute
            second = toFloat64(second)
          }
        }

        export fn makeTime(hm: Array<int64>, second: int64): Time {
          return new Time {
            hour = hm[0].toInt8()
            minute = hm[1].toInt8()
            second = second.toFloat64()
          }
        }

        export fn makeDateTime(date: Date, time: Time, timezone: HourMinute): DateTime {
          return new DateTime {
            date = date
            time = time
            timezone = timezone
          }
        }

        export fn makeDateTime(date: Date, time: Time): DateTime {
          return new DateTime {
            date = date
            time = time
            timezone = 00:00
          }
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: HourMinute): DateTime {
          return new DateTime {
            date = dt.date
            time = dt.time
            timezone = timezone
          }
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: Array<int64>): DateTime {
          return new DateTime {
            date = dt.date
            time = dt.time
            timezone = new HourMinute {
              hour = timezone[0].toInt8()
              minute = timezone[1].toInt8()
            }
          }
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: HourMinute): DateTime {
          return new DateTime {
            date = dt.date
            time = dt.time
            timezone = new HourMinute {
              hour = -timezone.hour
              minute = timezone.minute
            }
          }
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: Array<int64>): DateTime {
          return new Datetime {
            date = dt.date
            time = dt.time
            timezone = new HourMinute {
              hour = -toInt8(timezone[0])
              minute = toInt8(timezone[1])
            }
          }
        }

        // TODO: This should be in the root scope as an opcode
        fn abs(n: int8): int8 {
          if n < toInt8(0) {
            return -n
          } else {
            return n
          }
        }

        export fn print(dt: DateTime) {
          // TODO: Work on formatting stuff
          const timezoneOffsetSymbol = dt.timezone.hour < toInt8(0) ? \"-\" : \"+\"
          let str = (new Array<string> [
            toString(dt.date.year), \"-\", toString(dt.date.month), \"-\", toString(dt.date.day), \"@\",
            toString(dt.time.hour), \":\", toString(dt.time.minute), \":\", toString(dt.time.second),
            timezoneOffsetSymbol, abs(dt.timezone.hour).toString(), \":\", toString(dt.timezone.minute)
          ]).join('')
          print(str)
        }

        export prefix makeYear as # precedence 2
        export infix makeYearMonth as - precedence 2
        export infix makeDate as - precedence 2
        export infix makeHourMinute as : precedence 7
        export infix makeTime as : precedence 7
        export infix makeDateTime as @ precedence 2
        export infix makeDateTimeTimezone as + precedence 2
        export infix makeDateTimeTimezoneRev as - precedence 2

        export interface datetime {
          # int64: Year
          Year - int64: YearMonth
          YearMonth - int64: Date
          int64 : int64: HourMinute
          HourMinute : int64: Time
          Date @ Time: DateTime
          DateTime + HourMinute: DateTime
          DateTime - HourMinute: DateTime
          print(DateTime): void
        }
      "

      sourceToAll "
        from @std/app import start, print, exit
        from ./datetime import datetime

        on start {
          const dt = #2020-07-02@12:07:30-08:00
          dt.print()
          emit exit 0
        }
      "
    }
    BeforeAll before

    after() {
      cleanFile datetime.ln
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run node temp.js
      The output should eq "2020-7-2@12:7:30-8:0"
    End

    It "runs agc"
      When run alan run temp.agc
      The output should eq "2020-7-2@12:7:30-8:0"
    End
  End
End
