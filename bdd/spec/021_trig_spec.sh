Include build_tools.sh

Describe "@std/trig"
  before() {
    sourceToAll "
      from @std/app import start, print, exit
      import @std/trig
      from @std/trig import e, pi, tau
      // shouldn't be necessary, but compiler issue makes it so

      on start {
        'Logarithms and e^x'.print();
        print(trig.exp(e));
        print(trig.ln(e));
        print(trig.log(e));

        'Basic Trig functions'.print();
        print(trig.sin(tau / 6.0));
        print(trig.cos(tau / 6.0));
        print(trig.tan(tau / 6.0));
        print(trig.sec(tau / 6.0));
        print(trig.csc(tau / 6.0));
        print(trig.cot(tau / 6.0));

        'Inverse Trig functions'.print();
        print(trig.arcsine(0.0));
        print(trig.arccosine(1.0));
        print(trig.arctangent(0.0));
        print(trig.arcsecant(tau / 6.0));
        print(trig.arccosecant(tau / 6.0));
        print(trig.arccotangent(tau / 6.0));

        'Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )'.print();
        print(trig.versine(pi / 3.0));
        print(trig.vercosine(pi / 3.0));
        print(trig.coversine(pi / 3.0));
        print(trig.covercosine(pi / 3.0));
        print(trig.haversine(pi / 3.0));
        print(trig.havercosine(pi / 3.0));
        print(trig.hacoversine(pi / 3.0));
        print(trig.hacovercosine(pi / 3.0));
        print(trig.exsecant(pi / 3.0));
        print(trig.excosecant(pi / 3.0));
        print(trig.chord(pi / 3.0));

        'Historic Inverse Trig functions'.print();
        print(trig.aver(0.0));
        print(trig.avcs(0.5));
        print(trig.acvs(1.0));
        print(trig.acvc(1.0));
        print(trig.ahav(0.5));
        print(trig.ahvc(0.5));
        print(trig.ahcv(0.5));
        print(trig.ahcc(0.5));
        print(trig.aexs(0.5));
        print(trig.aexc(0.5));
        print(trig.acrd(0.5));

        'Hyperbolic Trig functions'.print();
        print(trig.sinh(tau / 6.0));
        print(trig.cosh(tau / 6.0));
        print(trig.tanh(tau / 6.0));
        print(trig.sech(tau / 6.0));
        print(trig.csch(tau / 6.0));
        print(trig.coth(tau / 6.0));

        'Inverse Hyperbolic Trig functions'.print();
        print(trig.hyperbolicArcsine(tau / 6.0));
        print(trig.hyperbolicArccosine(tau / 6.0));
        print(trig.hyperbolicArctangent(tau / 6.0));
        print(trig.hyperbolicArcsecant(0.5));
        print(trig.hyperbolicArccosecant(tau / 6.0));
        print(trig.hyperbolicArccotangent(tau / 6.0));

        emit exit 0;
      }
    "
  }
  BeforeAll before

  after() {
    cleanTemp
  }
  AfterAll after

  # I would like to have a unified output for both, but Javascript rounds things very slightly
  # differently from Rust at the least significant bit for the floating point numbers

  It "runs js"
    When run test_js
    The output should eq "Logarithms and e^x
15.154262241479259
1
0.43429448190325176
Basic Trig functions
0.8660254037844386
0.5000000000000001
1.7320508075688767
1.9999999999999996
1.1547005383792517
0.577350269189626
Inverse Trig functions
0
0
0
0.3013736097452911
1.2694227170496055
0.7623475341648746
Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )
0.4999999999999999
1.5
0.1339745962155614
1.8660254037844386
0.24999999999999994
0.75
0.0669872981077807
0.9330127018922193
0.9999999999999996
0.15470053837925168
0.9999999999999999
Historic Inverse Trig functions
0
2.0943951023931957
0
0
1.5707963267948966
1.5707963267948966
0
0
0.8410686705679303
0.7297276562269663
0.5053605102841573
Hyperbolic Trig functions
1.2493670505239751
1.600286857702386
0.7807144353592677
0.6248879662960872
0.8004052928885931
1.2808780710450447
Inverse Hyperbolic Trig functions
0.9143566553928857
0.3060421086132653
1.8849425394276085
1.3169578969248166
0.849142301064006
1.8849425394276085"
  End

  It "runs agc"
    When run test_agc
    The output should eq "Logarithms and e^x
15.154262241479259
1
0.4342944819032518
Basic Trig functions
0.8660254037844386
0.5000000000000001
1.7320508075688767
1.9999999999999996
1.1547005383792517
0.577350269189626
Inverse Trig functions
0
0
0
0.3013736097452911
1.2694227170496055
0.7623475341648746
Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )
0.4999999999999999
1.5
0.1339745962155614
1.8660254037844386
0.24999999999999994
0.75
0.0669872981077807
0.9330127018922193
0.9999999999999996
0.15470053837925168
0.9999999999999999
Historic Inverse Trig functions
0
2.0943951023931957
0
0
1.5707963267948966
1.5707963267948966
0
0
0.8410686705679303
0.7297276562269663
0.5053605102841573
Hyperbolic Trig functions
1.2493670505239751
1.600286857702386
0.7807144353592677
0.6248879662960872
0.8004052928885931
1.2808780710450447
Inverse Hyperbolic Trig functions
0.9143566553928857
0.3060421086132653
1.8849425394276085
1.3169578969248166
0.8491423010640059
1.8849425394276085"

  End
End
